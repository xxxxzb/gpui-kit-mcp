use std::time::Duration;

use schemars::JsonSchema;
use serde::Deserialize;

use super::{
    BridgeResult, GpuiMcp, Json, Operation, Parameters, ToolRouter, Value, json, object_output,
    tool, tool_router,
};

const WINDOW_SETTLE_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_WINDOW_SIZE: f32 = 16_384.0;

#[derive(Debug, Deserialize, JsonSchema)]
struct ResizeWindowArgs {
    /// Target content width in GPUI logical pixels.
    width: f32,
    /// Target content height in GPUI logical pixels.
    height: f32,
}

#[tool_router(router = window_router)]
impl GpuiMcp {
    #[tool(
        description = "Resize the selected GPUI window's content area in logical pixels, then report the geometry the window actually reached. Use this to check responsive layouts at several viewport widths"
    )]
    async fn resize_window(
        &self,
        Parameters(args): Parameters<ResizeWindowArgs>,
    ) -> Result<Json<Value>, String> {
        if !args.width.is_finite()
            || !args.height.is_finite()
            || !(1.0..=MAX_WINDOW_SIZE).contains(&args.width)
            || !(1.0..=MAX_WINDOW_SIZE).contains(&args.height)
        {
            return Err(
                "window size must be finite and between 1 and 16384 logical pixels".to_owned(),
            );
        }
        self.ack(Operation::ResizeWindow {
            width: args.width,
            height: args.height,
        })
        .await?;
        // The window manager may clamp the request to its minimum size or refuse a
        // size while entering full screen. Report the geometry either way rather
        // than turning a clamped resize into an error.
        if let Err(error) = self.settle_pending(WINDOW_SETTLE_TIMEOUT).await {
            tracing::warn!(%error, "frame settlement after window resize failed");
        }
        let result = self.call(Operation::GetWindowGeometry).await?;
        let BridgeResult::WindowGeometry(geometry) = result else {
            return Err("bridge returned the wrong result for window geometry".to_owned());
        };
        Ok(object_output(json!({
            "ok": true,
            "action": "window_resized",
            "requested": {
                "width": args.width,
                "height": args.height,
            },
            "content_bounds": geometry.content_bounds,
            "scale_factor": geometry.scale_factor,
        })))
    }
}

pub(super) fn router() -> ToolRouter<GpuiMcp> {
    GpuiMcp::window_router()
}
