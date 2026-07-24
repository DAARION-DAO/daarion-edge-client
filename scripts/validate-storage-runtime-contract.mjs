import {
  existsSync,
  readFileSync,
  readdirSync,
  statSync,
} from "node:fs";
import path from "node:path";
import ts from "typescript";

const paths = {
  client: "src/lib/storageRuntimeClient.ts",
  card: "src/components/StorageRuntimeCard.tsx",
  app: "src/App.tsx",
  rustCommand: "src-tauri/src/runtime_store/commands.rs",
  rustTypes: "src-tauri/src/runtime_store/types.rs",
  rustRoot: "src-tauri/src/lib.rs",
  packageManifest: "package.json",
  tsConfig: "tsconfig.json",
};

const command = "get_storage_runtime_status";
const commandConstant = "storageRuntimeCommand";
const clientFunction = "getStorageRuntimeStatus";
const cardComponent = "StorageRuntimeCard";
const tauriCoreModule = "@tauri-apps/api/core";
const frontendExtensions = new Set([".ts", ".tsx", ".js", ".jsx"]);

const approvedTauriCoreImporters = new Set([
  "src/App.tsx",
  "src/components/EdgeActivation.tsx",
  "src/components/GenesisWizard.tsx",
  "src/components/LocalModelsPanel.tsx",
  "src/components/MessagingPanel.tsx",
  "src/components/PairingGate.tsx",
  "src/lib/backendConfig.ts",
  "src/lib/inferenceClient.ts",
  "src/lib/storageRuntimeClient.ts",
]);

const approvedAdapterRuntimeExports = new Set([
  "StorageRuntimeClientError",
  "getStorageRuntimeStatus",
]);

const approvedAdapterTypeExports = new Set([
  "DatabaseHealth",
  "PersistenceState",
  "StorageRuntimeErrorCode",
  "StorageRuntimeState",
  "StorageRuntimeStatus",
]);

const fixtureResults = {
  primary: { positive: [], negative: [] },
  defense: { positive: [], negative: [] },
};

function normalizeRepoPath(value) {
  return value.replaceAll(path.sep, "/").replace(/^\.\//, "");
}

function listFrontendFiles(root) {
  const files = [];
  const walk = (directory) => {
    for (const entry of readdirSync(directory).sort()) {
      const absolute = path.join(directory, entry);
      if (statSync(absolute).isDirectory()) {
        walk(absolute);
        continue;
      }
      if (frontendExtensions.has(path.extname(entry))) {
        files.push(normalizeRepoPath(absolute));
      }
    }
  };
  walk(root);
  return files;
}

function readFrontendSources() {
  return new Map(
    listFrontendFiles("src").map((file) => [file, readFileSync(file, "utf8")]),
  );
}

function readSources() {
  return {
    frontend: readFrontendSources(),
    rustCommand: readFileSync(paths.rustCommand, "utf8"),
    rustTypes: readFileSync(paths.rustTypes, "utf8"),
    rustRoot: readFileSync(paths.rustRoot, "utf8"),
    runtimeStore: readdirSync("src-tauri/src/runtime_store")
      .filter((file) => file.endsWith(".rs"))
      .sort()
      .map((file) =>
        readFileSync(`src-tauri/src/runtime_store/${file}`, "utf8"),
      )
      .join("\n"),
    packageManifest: readFileSync(paths.packageManifest, "utf8"),
  };
}

const sources = readSources();

function cloneCandidate(candidate = sources) {
  return {
    ...candidate,
    frontend: new Map(candidate.frontend),
  };
}

function appendFrontend(candidate, file, source) {
  candidate.frontend.set(file, `${candidate.frontend.get(file) ?? ""}${source}`);
}

function replaceFrontend(candidate, file, search, replacement) {
  const source = candidate.frontend.get(file);
  if (source === undefined || !source.includes(search)) {
    throw new Error(`fixture could not find expected source in ${file}`);
  }
  candidate.frontend.set(file, source.replace(search, replacement));
}

function sourceFile(fileName, source) {
  const extension = path.extname(fileName);
  const kind =
    extension === ".tsx"
      ? ts.ScriptKind.TSX
      : extension === ".jsx"
        ? ts.ScriptKind.JSX
        : extension === ".js"
          ? ts.ScriptKind.JS
          : ts.ScriptKind.TS;
  return ts.createSourceFile(
    fileName,
    source,
    ts.ScriptTarget.Latest,
    true,
    kind,
  );
}

function visit(root, predicate) {
  const matches = [];
  const walk = (node) => {
    if (predicate(node)) matches.push(node);
    ts.forEachChild(node, walk);
  };
  walk(root);
  return matches;
}

function unwrapExpression(expression) {
  let current = expression;
  while (
    ts.isAsExpression(current) ||
    ts.isTypeAssertionExpression(current) ||
    ts.isParenthesizedExpression(current) ||
    ts.isSatisfiesExpression(current) ||
    ts.isNonNullExpression(current) ||
    ts.isAwaitExpression(current)
  ) {
    current = current.expression;
  }
  return current;
}

function hasExportModifier(node) {
  return Boolean(
    node.modifiers?.some(
      (modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword,
    ),
  );
}

function hasDefaultModifier(node) {
  return Boolean(
    node.modifiers?.some(
      (modifier) => modifier.kind === ts.SyntaxKind.DefaultKeyword,
    ),
  );
}

function namedBindingName(name) {
  if (ts.isIdentifier(name) || ts.isStringLiteral(name)) return name.text;
  return name.getText();
}

function readTsConfigAliases() {
  const read = ts.readConfigFile(paths.tsConfig, ts.sys.readFile);
  if (read.error) {
    return { baseUrl: ".", aliases: [], error: "tsconfig.json could not be parsed" };
  }
  const options = read.config?.compilerOptions ?? {};
  const baseUrl = normalizeRepoPath(options.baseUrl ?? ".");
  const aliases = [];
  for (const [pattern, targets] of Object.entries(options.paths ?? {})) {
    if (!Array.isArray(targets)) continue;
    aliases.push({ pattern, targets });
  }
  return { baseUrl, aliases, error: null };
}

const tsConfigAliases = readTsConfigAliases();

function aliasCandidates(specifier) {
  const candidates = [];
  for (const { pattern, targets } of tsConfigAliases.aliases) {
    const starIndex = pattern.indexOf("*");
    if (starIndex < 0) {
      if (specifier !== pattern) continue;
      for (const target of targets) {
        candidates.push(path.posix.normalize(path.posix.join(tsConfigAliases.baseUrl, target)));
      }
      continue;
    }
    if (pattern.indexOf("*", starIndex + 1) >= 0) continue;
    const prefix = pattern.slice(0, starIndex);
    const suffix = pattern.slice(starIndex + 1);
    if (!specifier.startsWith(prefix) || !specifier.endsWith(suffix)) continue;
    const matched = specifier.slice(prefix.length, specifier.length - suffix.length);
    for (const target of targets) {
      const targetStar = target.indexOf("*");
      candidates.push(
        path.posix.normalize(
          path.posix.join(
            tsConfigAliases.baseUrl,
            targetStar < 0
              ? target
              : `${target.slice(0, targetStar)}${matched}${target.slice(targetStar + 1)}`,
          ),
        ),
      );
    }
  }
  return candidates;
}

function moduleFileCandidates(base) {
  const extension = path.posix.extname(base);
  const withoutRuntimeExtension = [".js", ".jsx", ".mjs", ".cjs"].includes(extension)
    ? base.slice(0, -extension.length)
    : base;
  const candidates = [base, withoutRuntimeExtension];
  for (const value of [...candidates]) {
    for (const suffix of [".ts", ".tsx", ".js", ".jsx"]) {
      candidates.push(`${value}${suffix}`);
    }
    for (const suffix of [
      "/index.ts",
      "/index.tsx",
      "/index.js",
      "/index.jsx",
    ]) {
      candidates.push(`${value}${suffix}`);
    }
  }
  return [...new Set(candidates.map(normalizeRepoPath))];
}

function resolveLocalModule(importer, specifier, frontend) {
  const bases = [];
  let localIntent = false;
  if (specifier.startsWith(".")) {
    localIntent = true;
    bases.push(path.posix.normalize(path.posix.join(path.posix.dirname(importer), specifier)));
  } else {
    const aliases = aliasCandidates(specifier);
    if (aliases.length > 0) {
      localIntent = true;
      bases.push(...aliases);
    }
  }

  for (const base of bases) {
    for (const candidate of moduleFileCandidates(base)) {
      if (frontend.has(candidate)) return { target: candidate, unresolved: false };
    }
    if (existsSync(base) && !frontendExtensions.has(path.extname(base))) {
      return { target: null, unresolved: false };
    }
  }

  return { target: null, unresolved: localIntent };
}

function staticStringArgument(call) {
  if (call.arguments.length !== 1) return null;
  const argument = unwrapExpression(call.arguments[0]);
  if (
    ts.isStringLiteral(argument) ||
    ts.isNoSubstitutionTemplateLiteral(argument)
  ) {
    return argument.text;
  }
  return null;
}

function importIsTypeOnly(declaration, element = null) {
  return Boolean(
    declaration.importClause?.isTypeOnly ||
      (element && ts.isImportSpecifier(element) && element.isTypeOnly),
  );
}

function isExecutableCommandLiteral(node) {
  if (
    !(
      ts.isStringLiteral(node) ||
      ts.isNoSubstitutionTemplateLiteral(node)
    ) ||
    node.text !== command
  ) {
    return false;
  }
  if (ts.isLiteralTypeNode(node.parent)) return false;
  if (
    (ts.isImportDeclaration(node.parent) ||
      ts.isExportDeclaration(node.parent)) &&
    node.parent.moduleSpecifier === node
  ) {
    return false;
  }
  return true;
}

function rawExpression(expression, rawLocals) {
  const current = unwrapExpression(expression);
  if (ts.isIdentifier(current)) return rawLocals.has(current.text);
  if (ts.isPropertyAccessExpression(current)) {
    const owner = unwrapExpression(current.expression);
    return ts.isIdentifier(owner) && rawLocals.has(owner.text);
  }
  if (ts.isElementAccessExpression(current)) {
    const owner = unwrapExpression(current.expression);
    return ts.isIdentifier(owner) && rawLocals.has(owner.text);
  }
  if (ts.isConditionalExpression(current)) {
    return (
      rawExpression(current.whenTrue, rawLocals) ||
      rawExpression(current.whenFalse, rawLocals)
    );
  }
  return false;
}

function collectBindingIdentifiers(name) {
  if (ts.isIdentifier(name)) return [name.text];
  const names = [];
  for (const element of name.elements) {
    if (ts.isOmittedExpression(element)) continue;
    names.push(...collectBindingIdentifiers(element.name));
  }
  return names;
}

function parseFrontendModule(fileName, source, frontend) {
  const file = sourceFile(fileName, source);
  const meta = {
    fileName,
    file,
    directTauriImport: false,
    directTauriImportShapeValid: true,
    dynamicTauriImports: [],
    requireTauriImports: [],
    localImports: [],
    localExports: [],
    localReexports: [],
    directRawLocals: new Set(),
    rawLocals: new Set(),
    rawExports: new Set(),
    rawExportAll: false,
    exportedInitializers: [],
    exportedFunctions: [],
    runtimeExports: new Set(),
    typeExports: new Set(),
    commandLiterals: visit(file, isExecutableCommandLiteral),
    resolutionErrors: [],
    directTauriReexports: [],
  };

  for (const statement of file.statements) {
    if (ts.isImportDeclaration(statement)) {
      const specifier = ts.isStringLiteral(statement.moduleSpecifier)
        ? statement.moduleSpecifier.text
        : null;
      if (!specifier) continue;
      if (specifier === tauriCoreModule) {
        if (statement.importClause?.isTypeOnly) continue;
        meta.directTauriImport = true;
        const bindings = statement.importClause?.namedBindings;
        if (
          !bindings ||
          !ts.isNamedImports(bindings) ||
          statement.importClause?.name ||
          bindings.elements.length !== 1 ||
          importIsTypeOnly(statement, bindings.elements[0]) ||
          (bindings.elements[0].propertyName?.text ??
            bindings.elements[0].name.text) !== "invoke"
        ) {
          meta.directTauriImportShapeValid = false;
        }
        if (bindings && ts.isNamedImports(bindings)) {
          for (const element of bindings.elements) {
            if (
              !importIsTypeOnly(statement, element) &&
              (element.propertyName?.text ?? element.name.text) === "invoke"
            ) {
              meta.directRawLocals.add(element.name.text);
            }
          }
        } else if (bindings && ts.isNamespaceImport(bindings)) {
          meta.directRawLocals.add(bindings.name.text);
        }
        if (statement.importClause?.name) {
          meta.directRawLocals.add(statement.importClause.name.text);
        }
        continue;
      }

      const resolved = resolveLocalModule(fileName, specifier, frontend);
      if (resolved.unresolved) {
        meta.resolutionErrors.push(specifier);
        continue;
      }
      if (!resolved.target || !statement.importClause) continue;
      if (statement.importClause.name) {
        meta.localImports.push({
          target: resolved.target,
          importedName: "default",
          localName: statement.importClause.name.text,
          namespace: false,
        });
      }
      const bindings = statement.importClause.namedBindings;
      if (bindings && ts.isNamespaceImport(bindings)) {
        meta.localImports.push({
          target: resolved.target,
          importedName: "*",
          localName: bindings.name.text,
          namespace: true,
        });
      }
      if (bindings && ts.isNamedImports(bindings)) {
        for (const element of bindings.elements) {
          if (importIsTypeOnly(statement, element)) continue;
          meta.localImports.push({
            target: resolved.target,
            importedName: element.propertyName?.text ?? element.name.text,
            localName: element.name.text,
            namespace: false,
          });
        }
      }
      continue;
    }

    if (ts.isExportDeclaration(statement)) {
      const specifier =
        statement.moduleSpecifier && ts.isStringLiteral(statement.moduleSpecifier)
          ? statement.moduleSpecifier.text
          : null;
      if (!specifier) {
        if (statement.exportClause && ts.isNamedExports(statement.exportClause)) {
          for (const element of statement.exportClause.elements) {
            const exportedName = element.name.text;
            const localName = element.propertyName?.text ?? element.name.text;
            if (statement.isTypeOnly || element.isTypeOnly) {
              meta.typeExports.add(exportedName);
            } else {
              meta.runtimeExports.add(exportedName);
              meta.localExports.push({ localName, exportedName });
            }
          }
        }
        continue;
      }

      if (specifier === tauriCoreModule) {
        meta.directTauriReexports.push(statement);
        if (!statement.exportClause) {
          meta.rawExportAll = true;
        } else if (ts.isNamespaceExport(statement.exportClause)) {
          meta.rawExports.add(statement.exportClause.name.text);
        } else {
          for (const element of statement.exportClause.elements) {
            if (!statement.isTypeOnly && !element.isTypeOnly) {
              meta.rawExports.add(element.name.text);
            }
          }
        }
        continue;
      }

      const resolved = resolveLocalModule(fileName, specifier, frontend);
      if (resolved.unresolved) {
        meta.resolutionErrors.push(specifier);
        continue;
      }
      if (!resolved.target) continue;
      if (!statement.exportClause) {
        meta.localReexports.push({
          target: resolved.target,
          importedName: "*",
          exportedName: "*",
          wildcard: true,
        });
      } else if (ts.isNamespaceExport(statement.exportClause)) {
        meta.localReexports.push({
          target: resolved.target,
          importedName: "*",
          exportedName: statement.exportClause.name.text,
          wildcard: false,
        });
      } else {
        for (const element of statement.exportClause.elements) {
          if (statement.isTypeOnly || element.isTypeOnly) {
            meta.typeExports.add(element.name.text);
            continue;
          }
          meta.runtimeExports.add(element.name.text);
          meta.localReexports.push({
            target: resolved.target,
            importedName: element.propertyName?.text ?? element.name.text,
            exportedName: element.name.text,
            wildcard: false,
          });
        }
      }
      continue;
    }

    if (ts.isExportAssignment(statement)) {
      meta.runtimeExports.add("default");
      meta.exportedInitializers.push({
        exportedName: "default",
        initializer: statement.expression,
      });
      continue;
    }

    if (ts.isVariableStatement(statement) && hasExportModifier(statement)) {
      for (const declaration of statement.declarationList.declarations) {
        for (const name of collectBindingIdentifiers(declaration.name)) {
          meta.runtimeExports.add(name);
          if (declaration.initializer) {
            meta.exportedInitializers.push({
              exportedName: name,
              initializer: declaration.initializer,
            });
          }
        }
      }
      continue;
    }

    if (
      (ts.isFunctionDeclaration(statement) ||
        ts.isClassDeclaration(statement)) &&
      hasExportModifier(statement)
    ) {
      const exportedName = hasDefaultModifier(statement)
        ? "default"
        : statement.name?.text;
      if (exportedName) meta.runtimeExports.add(exportedName);
      if (ts.isFunctionDeclaration(statement) && exportedName) {
        meta.exportedFunctions.push({ exportedName, declaration: statement });
      }
      continue;
    }

    if (
      (ts.isInterfaceDeclaration(statement) ||
        ts.isTypeAliasDeclaration(statement)) &&
      hasExportModifier(statement)
    ) {
      meta.typeExports.add(statement.name.text);
    }
  }

  for (const call of visit(file, ts.isCallExpression)) {
    const specifier = staticStringArgument(call);
    if (specifier !== tauriCoreModule) continue;
    if (call.expression.kind === ts.SyntaxKind.ImportKeyword) {
      meta.dynamicTauriImports.push(call);
    } else if (
      ts.isIdentifier(call.expression) &&
      call.expression.text === "require"
    ) {
      meta.requireTauriImports.push(call);
    }
  }

  meta.rawLocals = new Set(meta.directRawLocals);
  return meta;
}

function propagateRawExports(modules) {
  let changed = true;
  while (changed) {
    changed = false;
    for (const meta of modules.values()) {
      for (const imported of meta.localImports) {
        const target = modules.get(imported.target);
        if (!target) continue;
        const isRaw = imported.namespace
          ? target.rawExportAll || target.rawExports.size > 0
          : target.rawExportAll || target.rawExports.has(imported.importedName);
        if (isRaw && !meta.rawLocals.has(imported.localName)) {
          meta.rawLocals.add(imported.localName);
          changed = true;
        }
      }

      for (const exported of meta.localExports) {
        if (
          meta.rawLocals.has(exported.localName) &&
          !meta.rawExports.has(exported.exportedName)
        ) {
          meta.rawExports.add(exported.exportedName);
          changed = true;
        }
      }

      for (const exported of meta.localReexports) {
        const target = modules.get(exported.target);
        if (!target) continue;
        if (exported.wildcard) {
          if (target.rawExportAll && !meta.rawExportAll) {
            meta.rawExportAll = true;
            changed = true;
          }
          for (const rawName of target.rawExports) {
            if (!meta.rawExports.has(rawName)) {
              meta.rawExports.add(rawName);
              changed = true;
            }
          }
          continue;
        }
        const isRaw =
          target.rawExportAll ||
          (exported.importedName === "*"
            ? target.rawExports.size > 0
            : target.rawExports.has(exported.importedName));
        if (isRaw && !meta.rawExports.has(exported.exportedName)) {
          meta.rawExports.add(exported.exportedName);
          changed = true;
        }
      }

      for (const exported of meta.exportedInitializers) {
        if (
          rawExpression(exported.initializer, meta.rawLocals) &&
          !meta.rawExports.has(exported.exportedName)
        ) {
          meta.rawExports.add(exported.exportedName);
          changed = true;
        }
      }

      for (const exported of meta.exportedFunctions) {
        const returnsRaw = visit(
          exported.declaration,
          (node) =>
            ts.isReturnStatement(node) &&
            node.expression !== undefined &&
            rawExpression(node.expression, meta.rawLocals),
        ).length;
        if (returnsRaw > 0 && !meta.rawExports.has(exported.exportedName)) {
          meta.rawExports.add(exported.exportedName);
          changed = true;
        }
      }
    }
  }
}

function analyzeFrontend(frontend) {
  const errors = [];
  const modules = new Map(
    [...frontend.entries()].map(([fileName, source]) => [
      fileName,
      parseFrontendModule(fileName, source, frontend),
    ]),
  );
  propagateRawExports(modules);

  const actualImporters = new Set(
    [...modules.values()]
      .filter((meta) => meta.directTauriImport)
      .map((meta) => meta.fileName),
  );
  if (!sameMembers([...actualImporters], [...approvedTauriCoreImporters])) {
    errors.push(
      "PRIMARY_MODULE_BOUNDARY_GATE: Tauri core importer baseline mismatch",
    );
  }

  for (const meta of modules.values()) {
    if (meta.directTauriImport && !meta.directTauriImportShapeValid) {
      errors.push(
        `PRIMARY_MODULE_BOUNDARY_GATE: unsupported Tauri core import shape in ${meta.fileName}`,
      );
    }
    if (meta.dynamicTauriImports.length > 0) {
      errors.push(
        `PRIMARY_MODULE_BOUNDARY_GATE: dynamic Tauri core import is forbidden in ${meta.fileName}`,
      );
    }
    if (meta.requireTauriImports.length > 0) {
      errors.push(
        `PRIMARY_MODULE_BOUNDARY_GATE: require of Tauri core is forbidden in ${meta.fileName}`,
      );
    }
    if (meta.directTauriReexports.length > 0) {
      errors.push(
        `PRIMARY_MODULE_BOUNDARY_GATE: direct Tauri binding re-export is forbidden in ${meta.fileName}`,
      );
    }
    if (meta.rawExportAll || meta.rawExports.size > 0) {
      errors.push(
        `PRIMARY_MODULE_BOUNDARY_GATE: Tauri binding re-export is forbidden in ${meta.fileName}`,
      );
    }
    for (const specifier of meta.resolutionErrors) {
      errors.push(
        `PRIMARY_MODULE_BOUNDARY_GATE: local import/re-export could not be resolved from ${meta.fileName}: ${specifier}`,
      );
    }
  }

  const commandOwners = [...modules.values()]
    .filter((meta) => meta.commandLiterals.length > 0)
    .map((meta) => ({
      fileName: meta.fileName,
      count: meta.commandLiterals.length,
    }));
  if (
    commandOwners.length !== 1 ||
    commandOwners[0].fileName !== paths.client ||
    commandOwners[0].count !== 1
  ) {
    errors.push(
      "PRIMARY_MODULE_BOUNDARY_GATE: executable storage command must have exactly one approved frontend owner",
    );
  }

  return { errors, modules, actualImporters, commandOwners };
}

function variableDeclaration(file, name) {
  for (const statement of file.statements.filter(ts.isVariableStatement)) {
    for (const declaration of statement.declarationList.declarations) {
      if (ts.isIdentifier(declaration.name) && declaration.name.text === name) {
        return { statement, declaration };
      }
    }
  }
  return null;
}

function exportedFunction(file, name) {
  return (
    file.statements.find(
      (statement) =>
        ts.isFunctionDeclaration(statement) &&
        statement.name?.text === name &&
        hasExportModifier(statement),
    ) ?? null
  );
}

function directIdentifierCalls(root, name) {
  return visit(
    root,
    (node) =>
      ts.isCallExpression(node) &&
      ts.isIdentifier(unwrapExpression(node.expression)) &&
      unwrapExpression(node.expression).text === name,
  );
}

function importBindings(file, moduleName) {
  const bindings = new Map();
  for (const declaration of file.statements.filter(ts.isImportDeclaration)) {
    if (!ts.isStringLiteral(declaration.moduleSpecifier)) continue;
    if (declaration.moduleSpecifier.text !== moduleName) continue;
    const named = declaration.importClause?.namedBindings;
    if (!named || !ts.isNamedImports(named)) continue;
    for (const element of named.elements) {
      bindings.set(
        element.propertyName?.text ?? element.name.text,
        element.name.text,
      );
    }
  }
  return bindings;
}

function containsNode(parent, child) {
  return child.getStart() >= parent.getStart() && child.getEnd() <= parent.getEnd();
}

function tsUnion(file, name) {
  const declaration = file.statements.find(
    (statement) =>
      ts.isTypeAliasDeclaration(statement) && statement.name.text === name,
  );
  if (!declaration || !ts.isTypeAliasDeclaration(declaration)) return null;
  const members = ts.isUnionTypeNode(declaration.type)
    ? declaration.type.types
    : [declaration.type];
  const values = [];
  for (const member of members) {
    if (
      !ts.isLiteralTypeNode(member) ||
      !ts.isStringLiteral(member.literal)
    ) {
      return null;
    }
    values.push(member.literal.text);
  }
  return values;
}

function tsInterfaceFields(file, name) {
  const declaration = file.statements.find(
    (statement) =>
      ts.isInterfaceDeclaration(statement) && statement.name.text === name,
  );
  if (!declaration || !ts.isInterfaceDeclaration(declaration)) return null;
  return declaration.members
    .filter(ts.isPropertySignature)
    .map((member) => member.name?.getText(file))
    .filter(Boolean);
}

function objectKeys(file, name) {
  const found = variableDeclaration(file, name);
  const initializer =
    found?.declaration.initializer &&
    unwrapExpression(found.declaration.initializer);
  if (!initializer || !ts.isObjectLiteralExpression(initializer)) return null;
  return initializer.properties
    .filter(ts.isPropertyAssignment)
    .map((property) =>
      property.name.getText(file).replace(/^['"]|['"]$/g, ""),
    );
}

function sameMembers(left, right) {
  if (left === null || right === null || left.length !== right.length) return false;
  const sortedLeft = [...left].sort();
  const sortedRight = [...right].sort();
  return sortedLeft.every((value, index) => value === sortedRight[index]);
}

function rawStringEnd(source, index) {
  let markerStart = -1;
  if (source[index] === "r") markerStart = index + 1;
  if (source[index] === "b" && source[index + 1] === "r") markerStart = index + 2;
  if (source[index] === "c" && source[index + 1] === "r") markerStart = index + 2;
  if (markerStart < 0) return null;
  let cursor = markerStart;
  while (source[cursor] === "#") cursor += 1;
  if (source[cursor] !== '"') return null;
  const hashes = source.slice(markerStart, cursor);
  const terminator = `"${hashes}`;
  const end = source.indexOf(terminator, cursor + 1);
  return end < 0 ? source.length : end + terminator.length;
}

function rustTokens(source) {
  const tokens = [];
  let index = 0;
  while (index < source.length) {
    const current = source[index];
    const next = source[index + 1];
    if (/\s/.test(current)) {
      index += 1;
      continue;
    }
    if (current === "/" && next === "/") {
      index = source.indexOf("\n", index + 2);
      if (index < 0) break;
      continue;
    }
    if (current === "/" && next === "*") {
      let depth = 1;
      index += 2;
      while (index < source.length && depth > 0) {
        if (source[index] === "/" && source[index + 1] === "*") {
          depth += 1;
          index += 2;
        } else if (source[index] === "*" && source[index + 1] === "/") {
          depth -= 1;
          index += 2;
        } else {
          index += 1;
        }
      }
      continue;
    }
    const rawEnd = rawStringEnd(source, index);
    if (rawEnd !== null) {
      index = rawEnd;
      continue;
    }
    if (
      current === '"' ||
      ((current === "b" || current === "c") && next === '"')
    ) {
      index += current === '"' ? 1 : 2;
      while (index < source.length) {
        if (source[index] === "\\") index += 2;
        else if (source[index] === '"') {
          index += 1;
          break;
        } else index += 1;
      }
      continue;
    }
    if (current === "'") {
      let cursor = index + 1;
      if (source[cursor] === "\\") cursor += 2;
      else cursor += 1;
      if (source[cursor] === "'") {
        index = cursor + 1;
        continue;
      }
      tokens.push("'");
      index += 1;
      continue;
    }
    if (/[A-Za-z_]/.test(current)) {
      let end = index + 1;
      while (/[A-Za-z0-9_]/.test(source[end] ?? "")) end += 1;
      tokens.push(source.slice(index, end));
      index = end;
      continue;
    }
    let matched = false;
    for (const compound of ["::", "->", "=>"]) {
      if (source.startsWith(compound, index)) {
        tokens.push(compound);
        index += compound.length;
        matched = true;
        break;
      }
    }
    if (matched) continue;
    tokens.push(current);
    index += 1;
  }
  return tokens;
}

function sequenceAt(tokens, index, sequence) {
  return sequence.every((token, offset) => tokens[index + offset] === token);
}

function matchingDelimiter(tokens, start, open, close) {
  let depth = 0;
  for (let index = start; index < tokens.length; index += 1) {
    if (tokens[index] === open) depth += 1;
    if (tokens[index] === close) {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return -1;
}

function snakeCase(value) {
  return value.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase();
}

function rustEnum(tokens, name) {
  for (let index = 0; index < tokens.length - 2; index += 1) {
    if (!sequenceAt(tokens, index, ["enum", name, "{"])) continue;
    const end = matchingDelimiter(tokens, index + 2, "{", "}");
    if (end < 0) return null;
    const variants = [];
    let depth = 0;
    for (let cursor = index + 3; cursor < end; cursor += 1) {
      if (["(", "{", "["].includes(tokens[cursor])) depth += 1;
      if ([")", "}", "]"].includes(tokens[cursor])) depth -= 1;
      if (
        depth === 0 &&
        /^[A-Z][A-Za-z0-9]*$/.test(tokens[cursor]) &&
        [",", "=", "(", "{"].includes(tokens[cursor + 1])
      ) {
        variants.push(snakeCase(tokens[cursor]));
      }
    }
    return variants;
  }
  return null;
}

function rustStructFields(tokens, name) {
  for (let index = 0; index < tokens.length - 2; index += 1) {
    if (!sequenceAt(tokens, index, ["struct", name, "{"])) continue;
    const end = matchingDelimiter(tokens, index + 2, "{", "}");
    if (end < 0) return null;
    const fields = [];
    let depth = 0;
    for (let cursor = index + 3; cursor < end - 1; cursor += 1) {
      if (["(", "{", "[", "<"].includes(tokens[cursor])) depth += 1;
      if ([")", "}", "]", ">"].includes(tokens[cursor])) depth -= 1;
      if (
        depth === 0 &&
        /^[A-Za-z_][A-Za-z0-9_]*$/.test(tokens[cursor]) &&
        tokens[cursor + 1] === ":"
      ) {
        fields.push(tokens[cursor]);
      }
    }
    return fields;
  }
  return null;
}

function tauriCommands(tokens) {
  const commands = [];
  const attribute = ["#", "[", "tauri", "::", "command", "]"];
  for (let index = 0; index < tokens.length; index += 1) {
    if (!sequenceAt(tokens, index, attribute)) continue;
    const fnIndex = tokens.indexOf("fn", index + attribute.length);
    if (fnIndex < 0 || fnIndex > index + attribute.length + 12) continue;
    const name = tokens[fnIndex + 1];
    const open = tokens.indexOf("(", fnIndex + 2);
    const close = open >= 0 ? matchingDelimiter(tokens, open, "(", ")") : -1;
    if (open < 0 || close < 0) continue;
    const parameters = [];
    let start = open + 1;
    let depth = 0;
    for (let cursor = open + 1; cursor <= close; cursor += 1) {
      if (["(", "[", "<"].includes(tokens[cursor])) depth += 1;
      if ([")", "]", ">"].includes(tokens[cursor])) depth -= 1;
      if ((tokens[cursor] === "," && depth === 0) || cursor === close) {
        const parameter = tokens.slice(start, cursor);
        if (parameter.length) parameters.push(parameter);
        start = cursor + 1;
      }
    }
    commands.push({ name, parameters });
  }
  return commands;
}

function injectedParameter(parameter) {
  const colon = parameter.indexOf(":");
  if (colon < 0) return false;
  const type = parameter
    .slice(colon + 1)
    .filter((token) => token !== "&" && token !== "'");
  return (
    sequenceAt(type, 0, ["tauri", "::", "AppHandle"]) ||
    sequenceAt(type, 0, ["tauri", "::", "State"])
  );
}

function handlerRegistrationCount(tokens, commandPath) {
  let count = 0;
  const startSequence = ["tauri", "::", "generate_handler", "!", "["];
  for (let index = 0; index < tokens.length; index += 1) {
    if (!sequenceAt(tokens, index, startSequence)) continue;
    const open = index + startSequence.length - 1;
    const close = matchingDelimiter(tokens, open, "[", "]");
    if (close < 0) continue;
    for (let cursor = open + 1; cursor < close; cursor += 1) {
      if (sequenceAt(tokens, cursor, commandPath)) count += 1;
    }
    index = close;
  }
  return count;
}

function validate(candidate) {
  const errors = [];
  let checks = 0;
  const require = (condition, message) => {
    checks += 1;
    if (!condition) errors.push(message);
  };

  require(
    tsConfigAliases.error === null,
    "PRIMARY_MODULE_BOUNDARY_GATE: tsconfig aliases could not be loaded",
  );

  const analysis = analyzeFrontend(candidate.frontend);
  for (const error of analysis.errors) {
    checks += 1;
    errors.push(error);
  }

  const clientSource = candidate.frontend.get(paths.client);
  const cardSource = candidate.frontend.get(paths.card);
  const appSource = candidate.frontend.get(paths.app);
  require(clientSource !== undefined, "approved storage adapter is missing");
  require(cardSource !== undefined, "Storage Runtime card is missing");
  require(appSource !== undefined, "App composition source is missing");
  if (!clientSource || !cardSource || !appSource) {
    return { errors, checks, analysis };
  }

  const client = sourceFile(paths.client, clientSource);
  const card = sourceFile(paths.card, cardSource);
  const app = sourceFile(paths.app, appSource);
  const clientMeta = analysis.modules.get(paths.client);
  const cardMeta = analysis.modules.get(paths.card);

  const constant = variableDeclaration(client, commandConstant);
  const constantInitializer =
    constant?.declaration.initializer &&
    unwrapExpression(constant.declaration.initializer);
  require(
    constant !== null &&
      !hasExportModifier(constant.statement) &&
      (constant.statement.declarationList.flags & ts.NodeFlags.Const) !== 0 &&
      constantInitializer &&
      ts.isStringLiteral(constantInitializer) &&
      constantInitializer.text === command,
    "PRIMARY_MODULE_BOUNDARY_GATE: storage command constant must be private and exact",
  );

  require(
    sameMembers(
      [...(clientMeta?.runtimeExports ?? [])],
      [...approvedAdapterRuntimeExports],
    ),
    "PRIMARY_MODULE_BOUNDARY_GATE: storage adapter runtime export surface changed",
  );
  require(
    sameMembers(
      [...(clientMeta?.typeExports ?? [])],
      [...approvedAdapterTypeExports],
    ),
    "PRIMARY_MODULE_BOUNDARY_GATE: storage adapter type export surface changed",
  );

  const clientExport = exportedFunction(client, clientFunction);
  require(clientExport !== null, "typed storage client function must be exported");
  require(
    clientExport?.parameters.length === 0,
    "typed storage client must accept no frontend path or SQL arguments",
  );
  const directInvokeCalls = clientExport
    ? directIdentifierCalls(clientExport, "invoke")
    : [];
  require(
    directInvokeCalls.length === 1,
    "SECONDARY_AST_DEFENSE_IN_DEPTH: typed client must contain exactly one direct invoke call",
  );
  require(
    directIdentifierCalls(client, "invoke").length === 1,
    "SECONDARY_AST_DEFENSE_IN_DEPTH: imported invoke must be used only by the direct typed client call",
  );
  if (directInvokeCalls.length === 1) {
    const call = directInvokeCalls[0];
    const argument = call.arguments[0] && unwrapExpression(call.arguments[0]);
    require(
      call.arguments.length === 1 &&
        argument !== undefined &&
        ((ts.isIdentifier(argument) && argument.text === commandConstant) ||
          (ts.isStringLiteral(argument) && argument.text === command)),
      "SECONDARY_AST_DEFENSE_IN_DEPTH: typed invoke must use only the private exact command",
    );
    require(
      clientExport !== null && containsNode(clientExport, call),
      "SECONDARY_AST_DEFENSE_IN_DEPTH: storage invoke must remain inside the typed client",
    );
  }

  const cardImports = importBindings(card, "../lib/storageRuntimeClient");
  const cardClientName = cardImports.get(clientFunction);
  require(
    cardClientName !== undefined,
    "PRIMARY_MODULE_BOUNDARY_GATE: Storage Runtime card must import the typed adapter",
  );
  require(
    cardClientName !== undefined &&
      directIdentifierCalls(card, cardClientName).length >= 1,
    "SECONDARY_AST_DEFENSE_IN_DEPTH: Storage Runtime card must call the typed adapter",
  );
  require(
    !cardMeta?.directTauriImport &&
      cardMeta?.dynamicTauriImports.length === 0 &&
      cardMeta?.requireTauriImports.length === 0 &&
      cardMeta?.commandLiterals.length === 0,
    "PRIMARY_MODULE_BOUNDARY_GATE: Storage Runtime card bypasses the typed adapter",
  );

  const appImports = importBindings(app, "./components/StorageRuntimeCard");
  const mountedName = appImports.get(cardComponent);
  require(
    mountedName !== undefined,
    "Dashboard must import the Storage Runtime card",
  );
  require(
    mountedName !== undefined &&
      visit(
        app,
        (node) =>
          (ts.isJsxSelfClosingElement(node) ||
            ts.isJsxOpeningElement(node)) &&
          ts.isIdentifier(node.tagName) &&
          node.tagName.text === mountedName,
      ).length >= 1,
    "Dashboard must structurally mount the Storage Runtime card",
  );

  const rustCommandTokens = rustTokens(candidate.rustCommand);
  const rustTypeTokens = rustTokens(candidate.rustTypes);
  const rustRootTokens = rustTokens(candidate.rustRoot);
  const runtimeTokens = rustTokens(
    `${candidate.runtimeStore}\n${candidate.rustCommand}`,
  );
  const commands = tauriCommands(rustCommandTokens);
  const exactCommands = commands.filter((item) => item.name === command);
  require(
    exactCommands.length === 1,
    "Rust storage status command is missing or duplicated",
  );
  require(
    exactCommands.length === 1 &&
      exactCommands[0].parameters.every(injectedParameter),
    "Rust storage status command must have no frontend-deserialized arguments",
  );
  require(
    handlerRegistrationCount(rustRootTokens, [
      "runtime_store",
      "::",
      "commands",
      "::",
      command,
    ]) === 1,
    "Rust storage status command must be registered exactly once",
  );

  for (const enumName of [
    "StorageRuntimeState",
    "StorageRuntimeErrorCode",
    "PersistenceState",
  ]) {
    require(
      sameMembers(rustEnum(rustTypeTokens, enumName), tsUnion(client, enumName)),
      `Rust and TypeScript ${enumName} variants must match exactly`,
    );
  }

  const tsFields = tsInterfaceFields(client, "StorageRuntimeStatus");
  const rustFields = rustStructFields(rustTypeTokens, "StorageRuntimeStatus");
  for (const field of [
    "state",
    "initialized",
    "schema_version",
    "database_health",
    "database_size_bytes",
    "storage_backend",
    "sqlite_version",
    "persistence_state",
    "last_start_time_ms",
    "error_code",
  ]) {
    require(
      tsFields?.includes(field) && rustFields?.includes(field),
      `frontend/Rust status field is missing: ${field}`,
    );
  }
  for (const forbidden of [
    "database_path",
    "sql_text",
    "raw_error",
    "connection_string",
  ]) {
    require(
      !tsFields?.includes(forbidden) && !rustFields?.includes(forbidden),
      `public status contract leaks forbidden field: ${forbidden}`,
    );
  }

  require(
    sameMembers(
      objectKeys(card, "stateLabels"),
      tsUnion(client, "StorageRuntimeState"),
    ),
    "Dashboard state labels must cover the exact runtime states",
  );
  const cardText = visit(
    card,
    (node) => ts.isStringLiteral(node) || ts.isJsxText(node),
  ).map((node) => node.text);
  for (const copy of [
    "Storage Runtime",
    "SQLite · Local device only",
    "No cloud sync",
  ]) {
    require(
      cardText.some((text) => text.includes(copy)),
      `required local-only UI copy is missing: ${copy}`,
    );
  }

  const forbiddenCommands = new Set([
    "execute_sql",
    "query_sql",
    "run_sql",
    "create_conversation",
    "append_message",
    "create_task",
    "append_audit_event",
    "delete_conversation",
  ]);
  const runtimeCommands = tauriCommands(runtimeTokens);
  require(
    runtimeCommands.every((item) => !forbiddenCommands.has(item.name)),
    "unauthorized storage CRUD or generic SQL command exists",
  );
  require(
    runtimeCommands.every((item) =>
      item.parameters.every((parameter) => {
        const colon = parameter.indexOf(":");
        return (
          injectedParameter(parameter) ||
          colon < 0 ||
          !parameter.slice(0, colon).includes("path")
        );
      }),
    ),
    "runtime_store must expose no Tauri command with a path argument",
  );

  require(
    visit(client, (node) => node.kind === ts.SyntaxKind.AnyKeyword).length === 0 &&
      visit(card, (node) => node.kind === ts.SyntaxKind.AnyKeyword).length === 0,
    "storage frontend contract must not use any",
  );
  let packageManifest = null;
  try {
    packageManifest = JSON.parse(candidate.packageManifest);
  } catch {
    // The structural assertion below reports the failure.
  }
  require(
    typeof packageManifest?.scripts?.["test:storage-runtime-contract"] ===
      "string",
    "required storage runtime contract package script is missing",
  );

  return { errors, checks, analysis };
}

function fixtureMustPass(group, label, mutate = () => {}) {
  const candidate = cloneCandidate();
  mutate(candidate);
  const result = validate(candidate);
  const accepted = result.errors.length === 0;
  fixtureResults[group].positive.push({ label, accepted });
  if (!accepted) {
    console.error(`FAIL: ${group} positive fixture rejected: ${label}`);
    for (const error of result.errors) console.error(`FAIL: ${label}: ${error}`);
  }
}

function mutationMustFail(group, label, mutate, expectedFragment) {
  const candidate = cloneCandidate();
  mutate(candidate);
  const result = validate(candidate);
  const rejected = result.errors.some((error) =>
    error.includes(expectedFragment),
  );
  fixtureResults[group].negative.push({ label, rejected });
  if (!rejected) {
    console.error(`FAIL: ${group} negative fixture accepted: ${label}`);
    for (const error of result.errors) console.error(`FAIL: ${label}: ${error}`);
  }
}

const positive = validate(sources);
for (const error of positive.errors) console.error(`FAIL: ${error}`);

fixtureMustPass("primary", "exact current repository source");
fixtureMustPass("primary", "audited legacy Tauri importer paths unchanged");
fixtureMustPass("primary", "private storage command constant");
fixtureMustPass("primary", "typed adapter call");
fixtureMustPass("primary", "normal Storage Runtime card consumption");
fixtureMustPass("primary", "unrelated local function named invoke", (candidate) => {
  appendFrontend(
    candidate,
    paths.card,
    "\nfunction invoke() { return undefined; }\nvoid invoke;\n",
  );
});
fixtureMustPass("primary", "ordinary object property named invoke", (candidate) => {
  appendFrontend(
    candidate,
    paths.card,
    "\nconst unrelatedBridge = { invoke: () => undefined };\nvoid unrelatedBridge.invoke;\n",
  );
});
fixtureMustPass("primary", "unrelated local import and re-export", (candidate) => {
  candidate.frontend.set(
    "src/lib/safeValue.ts",
    "export const safeValue = 1;\n",
  );
  candidate.frontend.set(
    "src/lib/safeValueBarrel.ts",
    'export { safeValue } from "./safeValue";\n',
  );
});
fixtureMustPass("primary", "safe typed application API barrel", (candidate) => {
  candidate.frontend.set(
    "src/lib/storageRuntimeBarrel.ts",
    'export { getStorageRuntimeStatus } from "./storageRuntimeClient";\nexport type { StorageRuntimeStatus } from "./storageRuntimeClient";\n',
  );
});

mutationMustFail(
  "primary",
  "new direct Tauri core importer",
  (candidate) => {
    candidate.frontend.set(
      "src/lib/unapprovedTauri.ts",
      `import { invoke } from "${tauriCoreModule}";\nvoid invoke;\n`,
    );
  },
  "importer baseline mismatch",
);
mutationMustFail(
  "primary",
  "direct Tauri core import in Storage Runtime card",
  (candidate) => {
    candidate.frontend.set(
      paths.card,
      `import { invoke } from "${tauriCoreModule}";\n${candidate.frontend.get(paths.card)}`,
    );
  },
  "importer baseline mismatch",
);
mutationMustFail(
  "primary",
  "dynamic Tauri core import in Storage Runtime card",
  (candidate) => {
    appendFrontend(
      candidate,
      paths.card,
      `\nvoid import("${tauriCoreModule}");\n`,
    );
  },
  "dynamic Tauri core import is forbidden",
);
mutationMustFail(
  "primary",
  "require of Tauri core in Storage Runtime card",
  (candidate) => {
    appendFrontend(
      candidate,
      paths.card,
      `\nvoid require("${tauriCoreModule}");\n`,
    );
  },
  "require of Tauri core is forbidden",
);
mutationMustFail(
  "primary",
  "storage command literal outside approved adapter",
  (candidate) => {
    appendFrontend(candidate, paths.card, `\nvoid "${command}";\n`);
  },
  "exactly one approved frontend owner",
);
mutationMustFail(
  "primary",
  "storage adapter exports imported invoke",
  (candidate) => {
    appendFrontend(candidate, paths.client, "\nexport { invoke };\n");
  },
  "Tauri binding re-export is forbidden",
);
mutationMustFail(
  "primary",
  "storage adapter exports Tauri namespace",
  (candidate) => {
    appendFrontend(
      candidate,
      paths.client,
      `\nimport * as tauriCore from "${tauriCoreModule}";\nexport { tauriCore };\n`,
    );
  },
  "unsupported Tauri core import shape",
);
mutationMustFail(
  "primary",
  "direct invoke re-export from Tauri core",
  (candidate) => {
    candidate.frontend.set(
      "src/lib/rawBarrel.ts",
      `export { invoke } from "${tauriCoreModule}";\n`,
    );
  },
  "direct Tauri binding re-export is forbidden",
);
mutationMustFail(
  "primary",
  "renamed invoke re-export from Tauri core",
  (candidate) => {
    candidate.frontend.set(
      "src/lib/rawBarrel.ts",
      `export { invoke as rawInvoke } from "${tauriCoreModule}";\n`,
    );
  },
  "direct Tauri binding re-export is forbidden",
);
mutationMustFail(
  "primary",
  "wildcard re-export from Tauri core",
  (candidate) => {
    candidate.frontend.set(
      "src/lib/rawBarrel.ts",
      `export * from "${tauriCoreModule}";\n`,
    );
  },
  "direct Tauri binding re-export is forbidden",
);
mutationMustFail(
  "primary",
  "local barrel exposes Tauri binding",
  (candidate) => {
    appendFrontend(candidate, paths.client, "\nexport { invoke };\n");
    candidate.frontend.set(
      "src/lib/rawBarrel.ts",
      'export { invoke } from "./storageRuntimeClient";\n',
    );
  },
  "Tauri binding re-export is forbidden",
);
mutationMustFail(
  "primary",
  "R4 assignment witness in Storage Runtime card",
  (candidate) => {
    candidate.frontend.set(
      paths.card,
      `import * as tauriCore from "${tauriCoreModule}";\n${candidate.frontend.get(paths.card)}\nlet rawInvoke: typeof tauriCore.invoke;\nrawInvoke = tauriCore.invoke;\nvoid rawInvoke("${command}");\n`,
    );
  },
  "importer baseline mismatch",
);
mutationMustFail(
  "primary",
  "R5 object-literal witness in Storage Runtime card",
  (candidate) => {
    candidate.frontend.set(
      paths.card,
      `import * as tauriCore from "${tauriCoreModule}";\n${candidate.frontend.get(paths.card)}\nconst rawInvokeHolder = { call: tauriCore.invoke };\nvoid rawInvokeHolder.call("${command}");\n`,
    );
  },
  "importer baseline mismatch",
);
mutationMustFail(
  "primary",
  "adapter command constant exported publicly",
  (candidate) => {
    replaceFrontend(
      candidate,
      paths.client,
      `const ${commandConstant} =`,
      `export const ${commandConstant} =`,
    );
  },
  "command constant must be private",
);
mutationMustFail(
  "primary",
  "typed adapter replaced by generic command execution",
  (candidate) => {
    replaceFrontend(
      candidate,
      paths.client,
      "export async function getStorageRuntimeStatus():",
      "export async function getStorageRuntimeStatus(commandName: string):",
    );
    replaceFrontend(
      candidate,
      paths.client,
      `invoke<StorageRuntimeStatus>(${commandConstant})`,
      "invoke<StorageRuntimeStatus>(commandName)",
    );
  },
  "no frontend path or SQL arguments",
);
mutationMustFail(
  "primary",
  "frontend path argument added to typed adapter",
  (candidate) => {
    replaceFrontend(
      candidate,
      paths.client,
      "export async function getStorageRuntimeStatus():",
      "export async function getStorageRuntimeStatus(path: string):",
    );
  },
  "no frontend path or SQL arguments",
);
mutationMustFail(
  "primary",
  "second executable storage command owner",
  (candidate) => {
    candidate.frontend.set(
      "src/lib/secondStorageOwner.ts",
      `export const duplicateCommand = "${command}";\n`,
    );
  },
  "exactly one approved frontend owner",
);
mutationMustFail(
  "primary",
  "raw storage invocation in grandfathered importer",
  (candidate) => {
    appendFrontend(candidate, paths.app, `\nvoid invoke("${command}");\n`);
  },
  "exactly one approved frontend owner",
);
mutationMustFail(
  "primary",
  "Rust storage command gains user argument",
  (candidate) => {
    candidate.rustCommand = candidate.rustCommand.replace(
      "app: tauri::AppHandle",
      "path: String, app: tauri::AppHandle",
    );
  },
  "no frontend-deserialized arguments",
);
mutationMustFail(
  "primary",
  "second public storage command registration",
  (candidate) => {
    candidate.rustRoot = candidate.rustRoot.replace(
      `runtime_store::commands::${command},`,
      `runtime_store::commands::${command},\n        runtime_store::commands::${command},`,
    );
  },
  "registered exactly once",
);

fixtureMustPass("defense", "ordinary local invoke function remains safe", (candidate) => {
  appendFrontend(
    candidate,
    paths.card,
    "\nfunction invoke() { return undefined; }\nvoid invoke();\n",
  );
});
fixtureMustPass("defense", "ordinary invoke property remains safe", (candidate) => {
  appendFrontend(
    candidate,
    paths.card,
    "\nconst safeObject = { invoke: () => undefined };\nvoid safeObject.invoke();\n",
  );
});
fixtureMustPass("defense", "safe typed barrel remains non-authoritative", (candidate) => {
  candidate.frontend.set(
    "src/lib/typedStorageBarrel.ts",
    'export { getStorageRuntimeStatus } from "./storageRuntimeClient";\n',
  );
});

mutationMustFail(
  "defense",
  "storage command constant renamed",
  (candidate) => {
    replaceFrontend(
      candidate,
      paths.client,
      `"${command}" as const`,
      '"renamed_storage_status" as const',
    );
  },
  "command constant must be private and exact",
);
mutationMustFail(
  "defense",
  "direct typed invoke removed",
  (candidate) => {
    replaceFrontend(
      candidate,
      paths.client,
      `return await invoke<StorageRuntimeStatus>(${commandConstant});`,
      'throw new Error("not implemented");',
    );
  },
  "exactly one direct invoke call",
);
mutationMustFail(
  "defense",
  "second direct invoke added to adapter",
  (candidate) => {
    appendFrontend(
      candidate,
      paths.client,
      `\nvoid invoke(${commandConstant});\n`,
    );
  },
  "used only by the direct typed client call",
);
mutationMustFail(
  "defense",
  "Storage Runtime card stops calling adapter",
  (candidate) => {
    replaceFrontend(
      candidate,
      paths.card,
      "setStatus(await getStorageRuntimeStatus());",
      "setStatus(null);",
    );
  },
  "must call the typed adapter",
);
mutationMustFail(
  "defense",
  "Dashboard unmounts Storage Runtime card",
  (candidate) => {
    replaceFrontend(candidate, paths.app, "<StorageRuntimeCard />", "<div />");
  },
  "must structurally mount",
);
mutationMustFail(
  "defense",
  "Rust command renamed",
  (candidate) => {
    candidate.rustCommand = candidate.rustCommand.replace(
      `fn ${command}`,
      "fn renamed_storage_status",
    );
  },
  "missing or duplicated",
);
mutationMustFail(
  "defense",
  "Rust registration removed",
  (candidate) => {
    candidate.rustRoot = candidate.rustRoot.replace(
      `runtime_store::commands::${command},`,
      `// runtime_store::commands::${command},`,
    );
  },
  "registered exactly once",
);
mutationMustFail(
  "defense",
  "Rust status variant removed",
  (candidate) => {
    candidate.rustTypes = candidate.rustTypes.replace("    Healthy,\n", "");
  },
  "StorageRuntimeState variants",
);
mutationMustFail(
  "defense",
  "frontend status field removed",
  (candidate) => {
    replaceFrontend(
      candidate,
      paths.client,
      "  sqlite_version: string | null;\n",
      "",
    );
  },
  "status field is missing",
);
mutationMustFail(
  "defense",
  "unauthorized storage CRUD command added",
  (candidate) => {
    candidate.rustCommand +=
      "\n#[tauri::command]\npub(crate) fn create_conversation() {}\n";
  },
  "unauthorized storage CRUD",
);

const groupsPass = Object.values(fixtureResults).every(
  (group) =>
    group.positive.every((fixture) => fixture.accepted) &&
    group.negative.every((fixture) => fixture.rejected),
);

if (positive.errors.length > 0 || !groupsPass) {
  process.exitCode = 1;
} else {
  const primary = fixtureResults.primary;
  const defense = fixtureResults.defense;
  const primaryPassed =
    primary.positive.filter((item) => item.accepted).length +
    primary.negative.filter((item) => item.rejected).length;
  const primaryTotal = primary.positive.length + primary.negative.length;
  const defensePassed =
    defense.positive.filter((item) => item.accepted).length +
    defense.negative.filter((item) => item.rejected).length;
  const defenseTotal = defense.positive.length + defense.negative.length;
  console.log(
    `PASS: PRIMARY_MODULE_BOUNDARY_GATE fixtures=${primaryPassed}/${primaryTotal} (${primary.positive.length} positive, ${primary.negative.length} negative); SECONDARY_AST_DEFENSE_IN_DEPTH fixtures=${defensePassed}/${defenseTotal} (${defense.positive.length} positive, ${defense.negative.length} negative); structural_checks=${positive.checks}; tauri_core_importers=${positive.analysis.actualImporters.size}; non_guarantees=ADR_0006`,
  );
  console.log(
    `TAURI_CORE_IMPORT_BASELINE: ${[...positive.analysis.actualImporters].sort().join(", ")}`,
  );
}
