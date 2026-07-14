import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const inferenceCommands = {
  status: "get_local_inference_status",
  models: "list_inference_models",
  prepare: "prepare_local_model",
  run: "run_local_inference",
  cancel: "cancel_local_inference",
  smoke: "run_local_inference_smoke",
} as const;

export interface ChatMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

export interface InferenceRequest {
  request_id: string;
  canonical_model_id: string;
  messages: ChatMessage[];
  max_tokens: number;
  temperature: number;
  stream: true;
}

export interface InferenceResponse {
  request_id: string;
  canonical_model_id: string;
  provider_id: string;
  latency_ms: number;
  output_text: string;
}

export interface InferenceStatus {
  execution_policy: "local_only";
  provider_id: string;
  endpoint: string;
  available: boolean;
  detail: string;
}

export interface InferenceModelSummary {
  canonical_model_id: string;
  family: string;
  tier: string;
  provider_id: string;
  installed: boolean;
}

export type InferenceEvent =
  | { kind: "started"; request_id: string }
  | { kind: "running"; request_id: string }
  | { kind: "token"; request_id: string; content: string }
  | { kind: "completed"; request_id: string }
  | { kind: "failed"; request_id: string; code: string; error: string }
  | { kind: "cancelled"; request_id: string }
  | { kind: "timed_out"; request_id: string };

interface InferenceCommandError {
  code: string;
  message: string;
}

async function invokeInference<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (reason) {
    if (
      typeof reason === "object" &&
      reason !== null &&
      "code" in reason &&
      "message" in reason &&
      typeof (reason as InferenceCommandError).code === "string" &&
      typeof (reason as InferenceCommandError).message === "string"
    ) {
      const error = reason as InferenceCommandError;
      throw new Error(`${error.code}: ${error.message}`);
    }
    throw new Error("local_inference_error: Local inference command failed");
  }
}

export function getLocalInferenceStatus(): Promise<InferenceStatus> {
  return invokeInference<InferenceStatus>(inferenceCommands.status);
}

export function listInferenceModels(): Promise<InferenceModelSummary[]> {
  return invokeInference<InferenceModelSummary[]>(inferenceCommands.models);
}

export function prepareLocalModel(canonicalModelId: string): Promise<void> {
  return invokeInference<void>(inferenceCommands.prepare, { canonicalModelId });
}

export function runLocalInference(request: InferenceRequest): Promise<InferenceResponse> {
  return invokeInference<InferenceResponse>(inferenceCommands.run, { request });
}

export function cancelLocalInference(requestId: string): Promise<boolean> {
  return invokeInference<boolean>(inferenceCommands.cancel, { requestId });
}

export function runLocalInferenceSmoke(canonicalModelId: string): Promise<InferenceResponse> {
  return invokeInference<InferenceResponse>(inferenceCommands.smoke, { canonicalModelId });
}

export function listenForInferenceEvents(
  handler: (event: InferenceEvent) => void,
): Promise<UnlistenFn> {
  return listen<InferenceEvent>("local-inference-event", (event) => handler(event.payload));
}
