use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingContext {
    pub priority: String,      // p1, p2, p3
    pub job_class: String,     // inf, idx, svc
    pub region: String,        // eu-w, us-e, global
    pub tier: String,          // t1, t2, t3
    pub specialization: String, // llm, emb, vis, any
}

pub struct RoutingLane;

impl RoutingLane {
    pub fn derive_subject(ctx: &RoutingContext) -> String {
        format!(
            "mm.{}.{}.{}.{}.{}",
            ctx.priority,
            ctx.job_class,
            ctx.region,
            ctx.tier,
            ctx.specialization
        )
    }

    pub fn to_subscription_filter(
        region: &str,
        tier: &str,
        specialization: &str
    ) -> String {
        // Example: mm.*.*.eu-w.t1.llm
        format!("mm.*.*.{}.{}.{}", region, tier, specialization)
    }
}
