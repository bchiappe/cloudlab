use leptos::prelude::*;
use crate::jobs::*;
use crate::components::icons::{List, CaretLeft};

#[component]
pub fn JobQueuePanel() -> impl IntoView {
    let (is_expanded, set_expanded) = signal(false);
    let (panel_height, _set_panel_height) = signal(300); // Adjustable in future
    
    let jobs_res = Resource::new(|| (), |_| async { list_jobs().await });
    
    // Auto-refresh every 5 seconds
    #[cfg(target_arch = "wasm32")]
    {
        leptos::prelude::Effect::new(move |_| {
            let _timer = gloo_timers::callback::Interval::new(5000, move || {
                jobs_res.refetch();
            });
        });
    }

    let active_jobs_count = move || {
        jobs_res.get().map(|res| {
            res.ok().map(|jobs| {
                jobs.iter().filter(|j| j.status == "running" || j.status == "pending").count()
            }).unwrap_or(0)
        }).unwrap_or(0)
    };

    view! {
        <div 
                class=move || format!(
                    "fixed bottom-0 right-0 left-0 bg-gray-900/95 backdrop-blur-md border-t border-gray-800 z-40 transition-all duration-300 ease-in-out shadow-2xl {}",
                    if is_expanded.get() { format!("h-[{}px]", panel_height.get()) } else { "h-12".into() }
                )
            >
                // Panel Header / Toggle
                <div 
                    on:click=move |_| set_expanded.update(|e| *e = !*e)
                    class="h-12 flex items-center justify-between px-6 cursor-pointer hover:bg-gray-800/50 transition-colors border-b border-gray-800/50"
                >
                    <div class="flex items-center gap-3">
                        <div class=move || format!("transition-colors {}", if active_jobs_count() > 0 { "text-blue-500 animate-pulse" } else { "text-gray-400" })>
                            <List class="w-5 h-5" />
                        </div>
                        <span class="text-sm font-bold text-gray-200 uppercase tracking-widest">"Active Tasks"</span>
                        <span class="px-2 py-0.5 bg-gray-800 rounded-full text-[10px] font-bold text-gray-400">
                            {move || active_jobs_count().to_string()}
                        </span>
                    </div>
                    
                    <div class="flex items-center gap-4">
                        <div class="hidden md:flex items-center gap-6 overflow-hidden max-w-[400px]">
                             <Suspense fallback=|| ()>
                                {move || jobs_res.get().map(|res| res.ok().map(|jobs| {
                                    jobs.iter().filter(|j| j.status == "running").take(1).map(|j| view! {
                                        <div class="flex items-center gap-3 animate-in fade-in slide-in-from-right-4">
                                            <span class="text-[10px] font-mono text-gray-500 truncate max-w-[150px]">{j.name.clone()}</span>
                                            <div class="w-32 h-1 bg-gray-800 rounded-full overflow-hidden">
                                                <div class="h-full bg-blue-500" style=format!("width: {}%", j.progress)></div>
                                            </div>
                                        </div>
                                    }).collect_view()
                                }))}
                             </Suspense>
                        </div>
                        <div class=move || format!("transition-transform duration-300 {}", if is_expanded.get() { "-rotate-90" } else { "rotate-90" })>
                            <CaretLeft class="w-4 h-4 text-gray-400" />
                        </div>
                    </div>
            </div>

            // Expanded Content
            <div class=move || format!("overflow-hidden h-full {}", if is_expanded.get() { "opacity-100" } else { "opacity-0 invisible h-0" })>
                <div class="p-6 h-[calc(100%-48px)] overflow-y-auto custom-scrollbar">
                    <Suspense fallback=|| view! { <div class="text-center py-12 text-gray-500">"Loading tasks..."</div> }>
                        {move || jobs_res.get().map(|res| match res {
                            Ok(jobs) if jobs.is_empty() => view! {
                                <div class="text-center py-12 text-gray-500 italic">"No recent background tasks."</div>
                            }.into_any(),
                            Ok(jobs) => view! {
                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                    {jobs.into_iter().take(6).map(|j| {
                                        let status_color = match j.status.as_str() {
                                            "running" => "text-blue-500",
                                            "completed" => "text-green-500",
                                            "failed" => "text-red-500",
                                            _ => "text-gray-500"
                                        };
                                        view! {
                                            <div class="p-4 bg-gray-950/50 rounded-xl border border-gray-800 group hover:border-gray-700 transition">
                                                <div class="flex justify-between items-start mb-3">
                                                    <span class="text-[10px] font-bold text-gray-400 uppercase tracking-widest">{j.name.clone()}</span>
                                                    <span class=format!("text-[10px] font-bold uppercase {}", status_color)>{j.status.clone()}</span>
                                                </div>
                                                <div class="space-y-2">
                                                    <div class="w-full h-1 bg-gray-800 rounded-full overflow-hidden text-blue-500">
                                                        <div 
                                                            class=format!("h-full transition-all duration-700 {}", if j.status == "running" { "bg-blue-500" } else if j.status == "completed" { "bg-green-500" } else { "bg-red-500" })
                                                            style=format!("width: {}%", j.progress)
                                                        ></div>
                                                    </div>
                                                    <div class="flex justify-between text-[10px] text-gray-600 font-mono">
                                                        <span>{j.started_at.clone()}</span>
                                                        <span>{j.progress}"%"</span>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any(),
                            Err(_) => view! { <div class="text-center py-12 text-red-400">"Failed to load job queue."</div> }.into_any(),
                        })}
                    </Suspense>
                    
                    <div class="mt-6 pt-6 border-t border-gray-800 flex justify-center">
                        <a href="/jobs" class="text-xs font-bold text-blue-500 hover:text-blue-400 transition uppercase tracking-widest">"View Full History →"</a>
                    </div>
                </div>
            </div>
        </div>
    }
}
