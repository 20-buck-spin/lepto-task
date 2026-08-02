use crate::models::sys_info::SysInfo;
use leptos::prelude::*;

#[component]
pub fn CpuInfoCard(sys_info: Signal<SysInfo>) -> impl IntoView {
    view!(
        <div class="cpu-info-container">
        {move || {
            sys_info.read().cpu_usage.iter().map(|cpu_usage| {
                let usage_clone = cpu_usage.clone();
                view!(
                    <div class="cpu-block"
                    style:background-color=move || {
                                    // Calculate brightness: 0 = dark, 100 = bright
                                    let brightness = usage_clone / 100.0; // 0.0 to 1.0

                                    // Interpolate between dark and bright colors
                                    // Starting with #0F172A (dark) and lightening it
                                    let r = (15 + (240 - 15)) as f32 * brightness; // 15 -> 240
                                    let g = (23 + (248 - 23)) as f32 * brightness; // 23 -> 248
                                    let b = (42 + (250 - 42)) as f32 * brightness; // 42 -> 250

                                    format!("rgb({}, {}, {})", r as u8, g as u8, b as u8)
                                }
                    ></div>
                )
            }).collect_view()
        }}
        </div>
    )
}
