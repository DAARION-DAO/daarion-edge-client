import { readFileSync, readdirSync } from "node:fs";
import ts from "typescript";

const paths = {
  client: "src/lib/storageRuntimeClient.ts",
  card: "src/components/StorageRuntimeCard.tsx",
  app: "src/App.tsx",
  rustCommand: "src-tauri/src/runtime_store/commands.rs",
  rustTypes: "src-tauri/src/runtime_store/types.rs",
  rustRoot: "src-tauri/src/lib.rs",
  packageManifest: "package.json",
};

const sources = Object.fromEntries(
  Object.entries(paths).map(([key, path]) => [key, readFileSync(path, "utf8")]),
);
sources.runtimeStore = readdirSync("src-tauri/src/runtime_store")
  .filter((file) => file.endsWith(".rs"))
  .map((file) => readFileSync(`src-tauri/src/runtime_store/${file}`, "utf8"))
  .join("\n");

const command = "get_storage_runtime_status";
const clientFunction = "getStorageRuntimeStatus";
const cardComponent = "StorageRuntimeCard";
const positiveFixtures = [];
const negativeFixtures = [];

function snakeCase(value) {
  return value.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase();
}

function sourceFile(name, source, jsx = false) {
  return ts.createSourceFile(
    name,
    source,
    ts.ScriptTarget.Latest,
    true,
    jsx ? ts.ScriptKind.TSX : ts.ScriptKind.TS,
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

function importBindings(file, moduleName) {
  const bindings = new Map();
  for (const declaration of file.statements.filter(ts.isImportDeclaration)) {
    if (!ts.isStringLiteral(declaration.moduleSpecifier)) continue;
    if (declaration.moduleSpecifier.text !== moduleName) continue;
    const named = declaration.importClause?.namedBindings;
    if (!named || !ts.isNamedImports(named)) continue;
    for (const element of named.elements) {
      bindings.set(element.propertyName?.text ?? element.name.text, element.name.text);
    }
  }
  return bindings;
}

function typedSource(file) {
  const fileName = `/storage-contract/${file.fileName}`;
  const compilerOptions = {
    target: ts.ScriptTarget.Latest,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    jsx: ts.JsxEmit.ReactJSX,
    strict: true,
    noLib: true,
    noResolve: true,
  };
  let programSource = null;
  const host = {
    fileExists: (requested) => requested === fileName,
    readFile: (requested) => (requested === fileName ? file.text : undefined),
    getSourceFile: (requested, languageVersion) => {
      if (requested !== fileName) return undefined;
      programSource ??= ts.createSourceFile(
        fileName,
        file.text,
        languageVersion,
        true,
        file.scriptKind,
      );
      return programSource;
    },
    getDefaultLibFileName: () => "/storage-contract/lib.d.ts",
    writeFile: () => {},
    getCurrentDirectory: () => "/storage-contract",
    getDirectories: () => [],
    getCanonicalFileName: (name) => name,
    useCaseSensitiveFileNames: () => true,
    getNewLine: () => "\n",
  };
  const program = ts.createProgram({
    rootNames: [fileName],
    options: compilerOptions,
    host,
  });
  return {
    file: program.getSourceFile(fileName),
    checker: program.getTypeChecker(),
  };
}

function symbolAtReference(expression, checker) {
  const current = unwrapExpression(expression);
  if (ts.isPropertyAccessExpression(current)) {
    return checker.getSymbolAtLocation(current.name) ?? checker.getSymbolAtLocation(current);
  }
  if (ts.isElementAccessExpression(current)) {
    const direct = checker.getSymbolAtLocation(current);
    if (direct) return direct;
    const property =
      current.argumentExpression && unwrapExpression(current.argumentExpression);
    if (
      property &&
      (ts.isStringLiteral(property) || ts.isNoSubstitutionTemplateLiteral(property))
    ) {
      return checker
        .getTypeAtLocation(current.expression)
        .getProperty(property.text);
    }
  }
  return checker.getSymbolAtLocation(current);
}

function hasSymbol(symbols, expression, checker) {
  const symbol = symbolAtReference(expression, checker);
  return symbol !== undefined && symbols.has(symbol);
}

function tauriCoreInvokeImports(file, checker) {
  const named = new Set();
  const namespaces = new Set();
  for (const declaration of file.statements.filter(ts.isImportDeclaration)) {
    if (!ts.isStringLiteral(declaration.moduleSpecifier)) continue;
    if (declaration.moduleSpecifier.text !== "@tauri-apps/api/core") continue;
    const bindings = declaration.importClause?.namedBindings;
    if (!bindings) continue;
    if (ts.isNamespaceImport(bindings)) {
      const symbol = checker.getSymbolAtLocation(bindings.name);
      if (symbol) namespaces.add(symbol);
      continue;
    }
    for (const element of bindings.elements) {
      if ((element.propertyName?.text ?? element.name.text) === "invoke") {
        const symbol = checker.getSymbolAtLocation(element.name);
        if (symbol) named.add(symbol);
      }
    }
  }
  return { named, namespaces };
}

function isTauriCoreDynamicImport(expression) {
  const current = unwrapExpression(expression);
  return (
    ts.isCallExpression(current) &&
    current.expression.kind === ts.SyntaxKind.ImportKeyword &&
    current.arguments.length === 1 &&
    ts.isStringLiteral(current.arguments[0]) &&
    current.arguments[0].text === "@tauri-apps/api/core"
  );
}

function flowExpressions(expression) {
  const current = unwrapExpression(expression);
  if (ts.isConditionalExpression(current)) {
    return [current.whenTrue, current.whenFalse];
  }
  if (ts.isBinaryExpression(current)) {
    const flowOperators = new Set([
      ts.SyntaxKind.EqualsToken,
      ts.SyntaxKind.CommaToken,
      ts.SyntaxKind.BarBarToken,
      ts.SyntaxKind.AmpersandAmpersandToken,
      ts.SyntaxKind.QuestionQuestionToken,
    ]);
    if (flowOperators.has(current.operatorToken.kind)) {
      return [current.left, current.right];
    }
  }
  return [];
}

function isTauriCoreNamespaceReference(expression, imports, checker) {
  const current = unwrapExpression(expression);
  if (hasSymbol(imports.namespaces, current, checker)) return true;
  if (isTauriCoreDynamicImport(current)) return true;
  return flowExpressions(current).some((part) =>
    isTauriCoreNamespaceReference(part, imports, checker),
  );
}

function isTauriInvokeReference(expression, imports, checker) {
  const current = unwrapExpression(expression);
  if (hasSymbol(imports.named, current, checker)) return true;
  if (ts.isPropertyAccessExpression(current)) {
    const owner = unwrapExpression(current.expression);
    if (
      isTauriCoreNamespaceReference(owner, imports, checker) &&
      current.name.text === "invoke"
    ) {
      return true;
    }
    return isTauriInvokeReference(current.expression, imports, checker);
  }
  if (ts.isElementAccessExpression(current)) {
    const owner = unwrapExpression(current.expression);
    if (isTauriCoreNamespaceReference(owner, imports, checker)) {
      const property =
        current.argumentExpression && unwrapExpression(current.argumentExpression);
      if (
        property &&
        (ts.isStringLiteral(property) || ts.isNoSubstitutionTemplateLiteral(property))
      ) {
        return property.text === "invoke";
      }
      // A dynamic namespace member could resolve to the privileged invoke binding.
      return true;
    }
    return isTauriInvokeReference(current.expression, imports, checker);
  }
  return flowExpressions(current).some((part) =>
    isTauriInvokeReference(part, imports, checker),
  );
}

function targetSymbols(target, checker) {
  const current = unwrapExpression(target);
  if (
    ts.isIdentifier(current) ||
    ts.isPropertyAccessExpression(current) ||
    ts.isElementAccessExpression(current)
  ) {
    const symbol = symbolAtReference(current, checker);
    return symbol ? [symbol] : [];
  }
  return [];
}

function destructuredInvokeTargets(target, checker) {
  const current = unwrapExpression(target);
  if (!ts.isObjectLiteralExpression(current)) return [];
  const targets = [];
  for (const property of current.properties) {
    if (ts.isShorthandPropertyAssignment(property)) {
      if (property.name.text === "invoke") {
        targets.push(...targetSymbols(property.name, checker));
      }
      continue;
    }
    if (!ts.isPropertyAssignment(property)) continue;
    const name = property.name;
    const couldBeInvoke =
      (ts.isIdentifier(name) && name.text === "invoke") ||
      (ts.isStringLiteral(name) && name.text === "invoke") ||
      ts.isComputedPropertyName(name);
    if (couldBeInvoke) {
      targets.push(...targetSymbols(property.initializer, checker));
    }
  }
  return targets;
}

function resolveTauriInvokeAliases(file, checker, imported) {
  const resolved = {
    named: new Set(imported.named),
    namespaces: new Set(imported.namespaces),
  };
  const declarations = visit(file, ts.isVariableDeclaration);
  const assignments = visit(
    file,
    (node) =>
      ts.isBinaryExpression(node) &&
      node.operatorToken.kind === ts.SyntaxKind.EqualsToken,
  );
  let changed = true;
  while (changed) {
    changed = false;
    for (const declaration of declarations) {
      const initializer = declaration.initializer && unwrapExpression(declaration.initializer);
      if (!initializer) continue;
      if (ts.isIdentifier(declaration.name)) {
        const targets = targetSymbols(declaration.name, checker);
        if (
          isTauriCoreNamespaceReference(initializer, resolved, checker)
        ) {
          for (const target of targets) {
            if (resolved.namespaces.has(target)) continue;
            resolved.namespaces.add(target);
            changed = true;
          }
          continue;
        }
        if (
          isTauriInvokeReference(initializer, resolved, checker)
        ) {
          for (const target of targets) {
            if (resolved.named.has(target)) continue;
            resolved.named.add(target);
            changed = true;
          }
        }
        continue;
      }
      if (
        ts.isObjectBindingPattern(declaration.name) &&
        isTauriCoreNamespaceReference(initializer, resolved, checker)
      ) {
        for (const element of declaration.name.elements) {
          if (!ts.isIdentifier(element.name)) continue;
          const property = element.propertyName ?? element.name;
          const couldBeInvoke =
            (ts.isIdentifier(property) && property.text === "invoke") ||
            (ts.isStringLiteral(property) && property.text === "invoke") ||
            ts.isComputedPropertyName(property);
          const symbol = checker.getSymbolAtLocation(element.name);
          if (couldBeInvoke && symbol && !resolved.named.has(symbol)) {
            resolved.named.add(symbol);
            changed = true;
          }
        }
      }
    }
    for (const assignment of assignments) {
      const targets = targetSymbols(assignment.left, checker);
      const assignedNamespace = isTauriCoreNamespaceReference(
        assignment.right,
        resolved,
        checker,
      );
      if (assignedNamespace) {
        for (const target of targets) {
          if (resolved.namespaces.has(target)) continue;
          resolved.namespaces.add(target);
          changed = true;
        }
        for (const target of destructuredInvokeTargets(assignment.left, checker)) {
          if (resolved.named.has(target)) continue;
          resolved.named.add(target);
          changed = true;
        }
      }
      if (isTauriInvokeReference(assignment.right, resolved, checker)) {
        for (const target of targets) {
          if (resolved.named.has(target)) continue;
          resolved.named.add(target);
          changed = true;
        }
      }
    }
  }
  return resolved;
}

function tauriInvokeCalls(file) {
  const typed = typedSource(file);
  const imports = resolveTauriInvokeAliases(
    typed.file,
    typed.checker,
    tauriCoreInvokeImports(typed.file, typed.checker),
  );
  const calls = visit(
    typed.file,
    (node) =>
      ts.isCallExpression(node) &&
      isTauriInvokeReference(node.expression, imports, typed.checker),
  );
  return { imports, calls };
}

function rawStorageInvokeCalls(file) {
  return tauriInvokeCalls(file).calls.filter((call) => {
    const commandArgument = call.arguments[0] && unwrapExpression(call.arguments[0]);
    if (
      commandArgument &&
      (ts.isStringLiteral(commandArgument) ||
        ts.isNoSubstitutionTemplateLiteral(commandArgument))
    ) {
      return commandArgument.text === command;
    }
    // A non-static command could resolve to the privileged storage command.
    return true;
  });
}

function containsNode(parent, child) {
  return child.getStart() >= parent.getStart() && child.getEnd() <= parent.getEnd();
}

function exportedVariableInitializer(file, name) {
  for (const statement of file.statements.filter(ts.isVariableStatement)) {
    const exported = statement.modifiers?.some(
      (modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword,
    );
    if (!exported) continue;
    for (const declaration of statement.declarationList.declarations) {
      if (ts.isIdentifier(declaration.name) && declaration.name.text === name) {
        return declaration.initializer ?? null;
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
        statement.modifiers?.some(
          (modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword,
        ),
    ) ?? null
  );
}

function tsUnion(file, name) {
  const declaration = file.statements.find(
    (statement) => ts.isTypeAliasDeclaration(statement) && statement.name.text === name,
  );
  if (!declaration || !ts.isTypeAliasDeclaration(declaration)) return null;
  const members = ts.isUnionTypeNode(declaration.type)
    ? declaration.type.types
    : [declaration.type];
  const values = [];
  for (const member of members) {
    if (!ts.isLiteralTypeNode(member) || !ts.isStringLiteral(member.literal)) return null;
    values.push(member.literal.text);
  }
  return values;
}

function tsInterfaceFields(file, name) {
  const declaration = file.statements.find(
    (statement) => ts.isInterfaceDeclaration(statement) && statement.name.text === name,
  );
  if (!declaration || !ts.isInterfaceDeclaration(declaration)) return null;
  return declaration.members
    .filter(ts.isPropertySignature)
    .map((member) => member.name?.getText(file))
    .filter(Boolean);
}

function objectKeys(file, name) {
  for (const statement of file.statements.filter(ts.isVariableStatement)) {
    for (const declaration of statement.declarationList.declarations) {
      if (!ts.isIdentifier(declaration.name) || declaration.name.text !== name) continue;
      const initializer = declaration.initializer && unwrapExpression(declaration.initializer);
      if (!initializer || !ts.isObjectLiteralExpression(initializer)) return null;
      return initializer.properties
        .filter(ts.isPropertyAssignment)
        .map((property) => property.name.getText(file).replace(/^['"]|['"]$/g, ""));
    }
  }
  return null;
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

// This lexer deliberately returns only code tokens. Comments and every Rust
// string/byte/character literal are discarded, so authority cannot be proved by
// text that the Rust parser would not execute. Nested block comments are handled.
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
    let compoundMatched = false;
    for (const compound of ["::", "->", "=>"]) {
      if (source.startsWith(compound, index)) {
        tokens.push(compound);
        index += compound.length;
        compoundMatched = true;
        break;
      }
    }
    if (compoundMatched) continue;
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
  const type = parameter.slice(colon + 1).filter((token) => token !== "&" && token !== "'");
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

  const client = sourceFile(paths.client, candidate.client);
  const card = sourceFile(paths.card, candidate.card, true);
  const app = sourceFile(paths.app, candidate.app, true);
  const clientInvoke = tauriInvokeCalls(client);
  const commandInitializer = exportedVariableInitializer(client, "storageRuntimeCommand");
  const literal = commandInitializer && unwrapExpression(commandInitializer);
  require(
    literal && ts.isStringLiteral(literal) && literal.text === command,
    "typed client command constant must be the exact executable string literal",
  );

  const clientExport = exportedFunction(client, clientFunction);
  require(clientExport !== null, "typed storage client function must be exported");
  require(
    clientExport?.parameters.length === 0,
    "typed storage client must accept no user path or SQL arguments",
  );
  require(
    clientInvoke.imports.named.size + clientInvoke.imports.namespaces.size > 0,
    "typed client must import the Tauri invoke binding",
  );
  require(
    clientInvoke.calls.length === 1,
    "typed client must have exactly one storage invoke binding",
  );
  require(
    clientExport !== null &&
      clientInvoke.calls.length === 1 &&
      containsNode(clientExport, clientInvoke.calls[0]),
    "storage invoke must remain inside the exported typed client",
  );
  if (clientInvoke.calls.length === 1 && ts.isCallExpression(clientInvoke.calls[0])) {
    const argument = clientInvoke.calls[0].arguments[0];
    require(
      clientInvoke.calls[0].arguments.length === 1 &&
        argument !== undefined &&
        ((ts.isIdentifier(argument) && argument.text === "storageRuntimeCommand") ||
          (ts.isStringLiteral(argument) && argument.text === command)),
      "typed client invoke must receive only the exact command constant or literal",
    );
  }

  const cardImports = importBindings(card, "../lib/storageRuntimeClient");
  const cardClientName = cardImports.get(clientFunction);
  require(
    cardClientName !== undefined,
    "Dashboard card must import the typed storage client",
  );
  require(
    visit(
      card,
      (node) =>
        ts.isCallExpression(node) &&
        ts.isIdentifier(node.expression) &&
        node.expression.text === cardClientName,
    ).length >= 1,
    "Dashboard card must call the typed storage client",
  );
  require(
    tauriInvokeCalls(card).calls.length === 0,
    "Dashboard card must contain no raw invoke",
  );

  const appImports = importBindings(app, "./components/StorageRuntimeCard");
  const mountedName = appImports.get(cardComponent);
  require(mountedName !== undefined, "Dashboard must import the storage runtime card");
  require(
    visit(
      app,
      (node) =>
        (ts.isJsxSelfClosingElement(node) || ts.isJsxOpeningElement(node)) &&
        ts.isIdentifier(node.tagName) &&
        node.tagName.text === mountedName,
    ).length >= 1,
    "existing Dashboard must structurally mount the storage runtime card",
  );
  require(
    rawStorageInvokeCalls(app).length === 0,
    "App must contain no raw storage invoke",
  );

  const rustCommandTokens = rustTokens(candidate.rustCommand);
  const rustTypeTokens = rustTokens(candidate.rustTypes);
  const rustRootTokens = rustTokens(candidate.rustRoot);
  const runtimeTokens = rustTokens(`${candidate.runtimeStore}\n${candidate.rustCommand}`);
  const lexicalProbe = rustTokens(`
    // #[tauri::command] ${command}
    /* outer /* #[tauri::command] ${command} */ still-commented */
    const NORMAL: &str = "#[tauri::command] ${command}";
    const RAW: &str = r###"#[tauri::command] ${command}"###;
    const BYTES: &[u8] = b"#[tauri::command] ${command}";
    const CHARACTER: char = 'g';
  `);
  require(
    !lexicalProbe.includes(command) &&
      !lexicalProbe.some((token, index) =>
        sequenceAt(lexicalProbe, index, ["#", "[", "tauri", "::", "command", "]"]),
      ),
    "Rust lexer must exclude nested comments, strings, raw strings, byte strings, and characters",
  );
  const commands = tauriCommands(rustCommandTokens);
  const exactCommands = commands.filter((item) => item.name === command);
  require(exactCommands.length === 1, "Rust storage status command is missing or duplicated");
  require(
    exactCommands.length === 1 && exactCommands[0].parameters.every(injectedParameter),
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

  const cardStates = objectKeys(card, "stateLabels");
  require(
    sameMembers(cardStates, tsUnion(client, "StorageRuntimeState")),
    "Dashboard state labels must cover the exact runtime states",
  );
  const cardText = visit(
    card,
    (node) => ts.isStringLiteral(node) || ts.isJsxText(node),
  ).map((node) => node.text);
  for (const copy of ["Storage Runtime", "SQLite · Local device only", "No cloud sync"]) {
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
      item.parameters.every(
        (parameter) =>
          injectedParameter(parameter) ||
          !parameter.slice(0, parameter.indexOf(":")).includes("path"),
      ),
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
    // The structural package check below reports the failure.
  }
  require(
    typeof packageManifest?.scripts?.["test:storage-runtime-contract"] === "string",
    "required storage runtime contract package script is missing",
  );
  return { errors, checks };
}

function mutationMustFail(label, mutate, expectedFragment) {
  const candidate = { ...sources };
  mutate(candidate);
  const { errors } = validate(candidate);
  const rejected = errors.some((error) => error.includes(expectedFragment));
  negativeFixtures.push({ label, rejected });
  if (!rejected) console.error(`FAIL: validator self-test did not reject ${label}`);
}

function fixtureMustPass(label, mutate) {
  const candidate = { ...sources };
  mutate(candidate);
  const { errors } = validate(candidate);
  const accepted = errors.length === 0;
  positiveFixtures.push({ label, accepted });
  if (!accepted) {
    console.error(`FAIL: validator self-test rejected safe fixture ${label}`);
    for (const error of errors) console.error(`FAIL: ${label}: ${error}`);
  }
}

const positive = validate(sources);
for (const error of positive.errors) console.error(`FAIL: ${error}`);

fixtureMustPass("approved typed storage client", () => {});
fixtureMustPass("unrelated ordinary function assignment", (candidate) => {
  candidate.card += `
function exerciseOrdinaryAssignment() {
  const ordinaryFunction = () => undefined;
  let assignedFunction: typeof ordinaryFunction;
  assignedFunction = ordinaryFunction;
  assignedFunction();
}
void exerciseOrdinaryAssignment;
`;
});
fixtureMustPass("local non-Tauri function named invoke", (candidate) => {
  candidate.card += `
function invoke() { return undefined; }
const localInvokeAlias = invoke;
void localInvokeAlias();
`;
});
fixtureMustPass("shadowed safe namespace identifier", (candidate) => {
  candidate.card =
    `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
    `
function exerciseSafeShadow() {
  const tauriCore = { invoke: () => undefined };
  let safeShadowAlias: typeof tauriCore.invoke;
  safeShadowAlias = tauriCore.invoke;
  safeShadowAlias();
}
void exerciseSafeShadow;
`;
});
fixtureMustPass("unused Tauri core namespace import", (candidate) => {
  candidate.card =
    `import * as unusedTauriCore from "@tauri-apps/api/core";\n${candidate.card}`;
});
fixtureMustPass("ordinary type annotations and formatting", (candidate) => {
  candidate.card += `
const formattedLocal = (value: string) => value;
let formattedAlias: typeof formattedLocal;
formattedAlias = ((formattedLocal as typeof formattedLocal)!);
void formattedAlias("safe");
`;
});
fixtureMustPass("unrelated object property named invoke", (candidate) => {
  candidate.card += `
const unrelatedBridge = { invoke: () => undefined };
let unrelatedPropertyAlias: typeof unrelatedBridge.invoke;
unrelatedPropertyAlias = unrelatedBridge.invoke;
void unrelatedPropertyAlias();
`;
});

mutationMustFail(
  "TypeScript command renamed with old command in a comment",
  (candidate) => {
    candidate.client = candidate.client.replace(
      `"${command}" as const`,
      `"renamed_storage_status" as const; // "${command}"`,
    );
  },
  "exact executable string literal",
);
mutationMustFail(
  "invoke binding changed with the old string elsewhere",
  (candidate) => {
    candidate.client = candidate.client.replace(
      "invoke<StorageRuntimeStatus>(storageRuntimeCommand)",
      `invoke<StorageRuntimeStatus>("renamed_storage_status") /* "${command}" */`,
    );
  },
  "exact command constant or literal",
);
mutationMustFail(
  "command constant moved into a comment",
  (candidate) => {
    candidate.client = candidate.client.replace(
      `export const storageRuntimeCommand = "${command}" as const;`,
      `// export const storageRuntimeCommand = "${command}" as const;`,
    );
  },
  "exact executable string literal",
);
mutationMustFail(
  "raw invoke added to StorageRuntimeCard",
  (candidate) => {
    candidate.card =
      `import { invoke } from "@tauri-apps/api/core";\n${candidate.card}` +
      `\nvoid invoke("${command}");\n`;
  },
  "no raw invoke",
);
mutationMustFail(
  "namespace invoke added to StorageRuntimeCard",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `\nvoid tauriCore.invoke("${command}");\n`;
  },
  "no raw invoke",
);
mutationMustFail(
  "renamed named invoke added to StorageRuntimeCard",
  (candidate) => {
    candidate.card =
      `import { invoke as tauriInvoke } from "@tauri-apps/api/core";\n${candidate.card}` +
      `\nvoid tauriInvoke("${command}");\n`;
  },
  "no raw invoke",
);
mutationMustFail(
  "namespace element invoke added to StorageRuntimeCard",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `\nvoid tauriCore["invoke"]("${command}");\n`;
  },
  "no raw invoke",
);
mutationMustFail(
  "namespace invoke added to App",
  (candidate) => {
    candidate.app =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.app}` +
      `\nvoid tauriCore.invoke("${command}");\n`;
  },
  "App must contain no raw storage invoke",
);
mutationMustFail(
  "namespace invoke receives a dynamic command",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      "\nconst dynamicStorageCommand = storageRuntimeCommand;\n" +
      "void tauriCore.invoke(dynamicStorageCommand);\n";
  },
  "no raw invoke",
);
mutationMustFail(
  "raw named invoke added to App",
  (candidate) => {
    candidate.app =
      `import { invoke } from "@tauri-apps/api/core";\n${candidate.app}` +
      `\nvoid invoke("${command}");\n`;
  },
  "App must contain no raw storage invoke",
);
mutationMustFail(
  "additional namespace invoke added to typed client",
  (candidate) => {
    candidate.client =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.client}` +
      `\nvoid tauriCore.invoke("${command}");\n`;
  },
  "exactly one storage invoke binding",
);
mutationMustFail(
  "namespace invoke alias added to StorageRuntimeCard",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `\nconst delegatedInvoke = tauriCore.invoke;\n` +
      `void delegatedInvoke("${command}");\n`;
  },
  "no raw invoke",
);
mutationMustFail(
  "dynamic core import invoke added to StorageRuntimeCard",
  (candidate) => {
    candidate.card +=
      `\nvoid (await import("@tauri-apps/api/core")).invoke("${command}");\n`;
  },
  "no raw invoke",
);
mutationMustFail(
  "aliased dynamic core import added to StorageRuntimeCard",
  (candidate) => {
    candidate.card +=
      `\nconst dynamicTauriCore = await import("@tauri-apps/api/core");\n` +
      `void dynamicTauriCore.invoke("${command}");\n`;
  },
  "no raw invoke",
);
mutationMustFail(
  "R4 assignment alias reproduction",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
let rawInvoke: typeof tauriCore.invoke;
rawInvoke = tauriCore.invoke;
void rawInvoke("${command}");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "assignment from direct named import",
  (candidate) => {
    candidate.card =
      `import { invoke } from "@tauri-apps/api/core";\n${candidate.card}` +
      `
let assignedNamedInvoke: typeof invoke;
assignedNamedInvoke = invoke;
void assignedNamedInvoke("${command}");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "assignment from renamed named import",
  (candidate) => {
    candidate.card =
      `import { invoke as importedInvoke } from "@tauri-apps/api/core";\n${candidate.card}` +
      `
let assignedRenamedInvoke: typeof importedInvoke;
assignedRenamedInvoke = importedInvoke;
void assignedRenamedInvoke("${command}");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "assignment from namespace element access",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
let assignedElementInvoke: typeof tauriCore.invoke;
assignedElementInvoke = tauriCore["invoke"];
void assignedElementInvoke("${command}");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "chained assignment alias",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
let chainedAliasOne: typeof tauriCore.invoke;
let chainedAliasTwo: typeof tauriCore.invoke;
chainedAliasTwo = chainedAliasOne = tauriCore.invoke;
void chainedAliasTwo("${command}");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "two-step assignment alias propagation",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
let firstAssignedAlias: typeof tauriCore.invoke;
let secondAssignedAlias: typeof tauriCore.invoke;
firstAssignedAlias = tauriCore.invoke;
secondAssignedAlias = firstAssignedAlias;
void secondAssignedAlias("${command}");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "parenthesized assignment right-hand side",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
let parenthesizedAlias: typeof tauriCore.invoke;
parenthesizedAlias = (tauriCore.invoke);
void parenthesizedAlias("${command}");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "as-asserted assignment right-hand side",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
let asAssertedAlias: typeof tauriCore.invoke;
asAssertedAlias = tauriCore.invoke as typeof tauriCore.invoke;
void asAssertedAlias("${command}");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "non-null assignment right-hand side",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
let nonNullAlias: typeof tauriCore.invoke;
nonNullAlias = tauriCore.invoke!;
void nonNullAlias("${command}");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "type-asserted assignment right-hand side",
  (candidate) => {
    candidate.client += `
let typeAssertedAlias: typeof invoke;
typeAssertedAlias = <typeof invoke>invoke;
void typeAssertedAlias("${command}");
`;
  },
  "exactly one storage invoke binding",
);
mutationMustFail(
  "assignment alias called inside nested function",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
let nestedFunctionAlias: typeof tauriCore.invoke;
nestedFunctionAlias = tauriCore.invoke;
function callAssignedAlias() {
  return nestedFunctionAlias("${command}");
}
void callAssignedAlias;
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "assignment alias called with dynamic command",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
let dynamicCommandAlias: typeof tauriCore.invoke;
dynamicCommandAlias = tauriCore.invoke;
const assignedCommand = "${command}";
void dynamicCommandAlias(assignedCommand);
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "unsafe outer symbol with unrelated shadowed safe identifier",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
let shadowedAlias: typeof tauriCore.invoke;
shadowedAlias = tauriCore.invoke;
function useSafeShadow() {
  const shadowedAlias = () => undefined;
  shadowedAlias();
}
void shadowedAlias("${command}");
void useSafeShadow;
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "assignment before var declaration",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
beforeDeclarationAlias = tauriCore.invoke;
var beforeDeclarationAlias: typeof tauriCore.invoke;
void beforeDeclarationAlias("${command}");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "assignment alias with old command only in comment",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
let commentProofAlias: typeof tauriCore.invoke;
commentProofAlias = tauriCore.invoke;
// ${command}
void commentProofAlias("renamed_storage_status");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "dynamic namespace property assignment",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
const invokePropertyName = "invoke";
let dynamicPropertyAlias: typeof tauriCore.invoke;
dynamicPropertyAlias = tauriCore[invokePropertyName];
void dynamicPropertyAlias("${command}");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "assignment into an object property alias",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
const propertyAliasHolder = { call: () => undefined };
propertyAliasHolder.call = tauriCore.invoke;
void propertyAliasHolder.call("${command}");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "assignment into an object element alias",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
const elementAliasHolder = { call: () => undefined };
elementAliasHolder["call"] = tauriCore.invoke;
void elementAliasHolder["call"]("${command}");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "destructuring assignment from Tauri core namespace",
  (candidate) => {
    candidate.card =
      `import * as tauriCore from "@tauri-apps/api/core";\n${candidate.card}` +
      `
let destructuredAssignmentAlias: typeof tauriCore.invoke;
({ invoke: destructuredAssignmentAlias } = tauriCore);
void destructuredAssignmentAlias("${command}");
`;
  },
  "no raw invoke",
);
mutationMustFail(
  "Rust command renamed with the old name in a line comment",
  (candidate) => {
    candidate.rustCommand = candidate.rustCommand.replace(
      `fn ${command}`,
      `fn renamed_storage_status // fn ${command}\n`,
    );
  },
  "missing or duplicated",
);
mutationMustFail(
  "Rust registration removed with the old registration in a comment",
  (candidate) => {
    candidate.rustRoot = candidate.rustRoot.replace(
      `runtime_store::commands::${command},`,
      `// runtime_store::commands::${command},`,
    );
  },
  "registered exactly once",
);
mutationMustFail(
  "Rust registration hidden in a nested block comment",
  (candidate) => {
    candidate.rustRoot = candidate.rustRoot.replace(
      `runtime_store::commands::${command},`,
      `/* outer /* nested */ runtime_store::commands::${command}, */`,
    );
  },
  "registered exactly once",
);
mutationMustFail(
  "duplicate Rust handler registration",
  (candidate) => {
    candidate.rustRoot = candidate.rustRoot.replace(
      `runtime_store::commands::${command},`,
      `runtime_store::commands::${command},\n        runtime_store::commands::${command},`,
    );
  },
  "registered exactly once",
);
mutationMustFail(
  "frontend path argument added",
  (candidate) => {
    candidate.rustCommand = candidate.rustCommand.replace(
      "app: tauri::AppHandle",
      "path: String, app: tauri::AppHandle",
    );
  },
  "no frontend-deserialized arguments",
);
mutationMustFail(
  "missing Rust state variant",
  (candidate) => {
    candidate.rustTypes = candidate.rustTypes.replace("    Healthy,\n", "");
  },
  "StorageRuntimeState variants",
);
mutationMustFail(
  "extra TypeScript error-code variant",
  (candidate) => {
    candidate.client = candidate.client.replace(
      '  | "internal";',
      '  | "internal"\n  | "unexpected";',
    );
  },
  "StorageRuntimeErrorCode variants",
);
mutationMustFail(
  "persistence-state mismatch",
  (candidate) => {
    candidate.client = candidate.client.replace('  | "created_new"\n', "");
  },
  "PersistenceState variants",
);
mutationMustFail(
  "storage CRUD command added",
  (candidate) => {
    candidate.rustCommand +=
      "\n#[tauri::command]\npub(crate) fn create_conversation() {}\n";
  },
  "CRUD or generic SQL",
);
mutationMustFail(
  "generic SQL command added",
  (candidate) => {
    candidate.rustCommand += "\n#[tauri::command]\npub(crate) fn execute_sql() {}\n";
  },
  "CRUD or generic SQL",
);

const rejected = negativeFixtures.filter((fixture) => fixture.rejected).length;
const accepted = positiveFixtures.filter((fixture) => fixture.accepted).length;
if (
  positive.errors.length ||
  accepted !== positiveFixtures.length ||
  rejected !== negativeFixtures.length
) {
  process.exitCode = 1;
} else {
  console.log(
    `PASS: structural storage runtime contract; positive checks=${positive.checks}; positive fixtures=${accepted}/${positiveFixtures.length}; negative fixtures=${rejected}/${negativeFixtures.length}`,
  );
}
