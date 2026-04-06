use crate::agents::optimization_plan::{OptimizationPlan, PlanAction};
use crate::agents::approval_proposal::{ApprovalProposal, ApprovalAction, ApprovalScope};

pub struct PlanExecutor;

impl PlanExecutor {
    pub fn convert_to_proposals(plan: &OptimizationPlan) -> Vec<ApprovalProposal> {
        plan.recommended_actions.iter().map(|action| {
             match action {
                 PlanAction::PromoteModel { model_id, role } => {
                     ApprovalProposal {
                         proposal_id: uuid::Uuid::new_v4().to_string(),
                         agent_id: "planning_orchestrator".to_string(),
                         target_model_id: model_id.clone(),
                         action: ApprovalAction::PromoteToWarm, // Mapping logic
                         scope: ApprovalScope::District,
                         reason_summary: format!("Plan {}: {}", plan.plan_id, plan.telemetry_basis),
                         confidence: plan.confidence,
                         telemetry_refs: vec![plan.plan_id.clone()],
                         trust_scope: "DistrictPlanner".to_string(),
                         created_at: chrono::Utc::now().timestamp_millis() as u64,
                     }
                 },
                 _ => {
                      // Placeholder for other mappings
                      ApprovalProposal {
                         proposal_id: uuid::Uuid::new_v4().to_string(),
                         agent_id: "planning_orchestrator".to_string(),
                         target_model_id: "unknown".to_string(),
                         action: ApprovalAction::AvoidPlacement,
                         scope: ApprovalScope::Node,
                         reason_summary: "Unsupported plan action".to_string(),
                         confidence: 0.0,
                         telemetry_refs: vec![],
                         trust_scope: "None".to_string(),
                         created_at: 0,
                      }
                 }
             }
        }).collect()
    }
}
