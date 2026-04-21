use leptos::prelude::*;
use crate::jobs::*;
use crate::components::icons::{List, ArrowCounterClockwise};

#[component]
pub fn JobsPage() -> impl IntoView {
    let jobs_res = Resource::new(|| (), |_| async { list_jobs().await });
    let clear_action = ServerAction::<ClearCompletedJobs>::new();
    let (selected_job_id, set_selected_job_id) = signal(Option::<String>::None);
    let (selected_job_name, set_selected_job_name) = signal(String::new());

    // Polling for logs in modal
    let (poll_trigger, _set_poll_trigger) = signal(0);
    #[cfg(target_arch = "wasm32")]
    let _ = {
        use gloo_timers::callback::Interval;
        let interval = Interval::new(2000, move || {
            _set_poll_trigger.update(|n| *n += 1);
        });
        interval
    };

    let job_logs_res = Resource::new(move || (selected_job_id.get(), poll_trigger.get()), |(jid, _)| async move {
        if let Some(id) = jid {
            crate::jobs::list_job_logs(id).await
        } else {
            Ok(vec![])
        }
    });

    view! {
        <div class="space-y-6 animate-in fade-in duration-500">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold tracking-tight text-gray-100 flex items-center gap-3">
                        <List class="w-8 h-8 text-blue-500"/>
                        "Background Jobs"
                    </h1>
                    <p class="text-gray-400 mt-1">"Monitor and manage asynchronous infrastructure tasks."</p>
                </div>
                // ... (refresh/clear buttons)
                <div class="flex items-center gap-3">
                    <button 
                         on:click=move |_| jobs_res.refetch()
                         class="p-2 text-gray-400 hover:text-white bg-gray-900 border border-gray-800 rounded-lg transition"
                         title="Refresh"
                    >
                        <ArrowCounterClockwise class="w-5 h-5"/>
                    </button>
                    <button 
                        on:click=move |_| {
                            clear_action.dispatch(ClearCompletedJobs {});
                            jobs_res.refetch();
                        }
                        class="px-4 py-2 border border-red-500/20 text-red-500 hover:bg-red-500/10 rounded-lg text-sm font-semibold transition cursor-pointer"
                    >
                        "Clear History"
                    </button>
                </div>
            </div>

            <div class="bg-gray-900/50 backdrop-blur-sm border border-gray-800 rounded-xl overflow-hidden shadow-2xl">
                <table class="w-full text-left border-collapse text-sm">
                    <thead>
                        <tr class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wider">
                            <th class="px-6 py-4 font-semibold">"Job Name"</th>
                            <th class="px-6 py-4 font-semibold">"Status"</th>
                            <th class="px-6 py-4 font-semibold">"Progress"</th>
                            <th class="px-6 py-4 font-semibold">"Started At"</th>
                            <th class="px-6 py-4 font-semibold text-right">"Logs"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-gray-800/50">
                        <Suspense fallback=|| view! { <tr><td colspan="5" class="px-6 py-8 text-center text-gray-500">"Loading job history..."</td></tr> }>
                            {move || jobs_res.get().map(|res| match res {
                                Ok(jobs) if jobs.is_empty() => view! {
                                    <tr><td colspan="5" class="px-6 py-12 text-center text-gray-500 italic">"No jobs found in history."</td></tr>
                                }.into_any(),
                                Ok(jobs) => jobs.into_iter().map(|job| {
                                    let id = job.id.clone();
                                    let name = job.name.clone();
                                    view! {
                                        <tr class="hover:bg-gray-800/30 transition-colors group">
                                            <td class="px-6 py-4">
                                                <div class="flex flex-col">
                                                    <span class="font-semibold text-gray-100 uppercase text-xs tracking-tight">{job.name}</span>
                                                    <span class="text-[10px] font-mono text-gray-500">{"ID: "} {job.id.chars().take(8).collect::<String>()}</span>
                                                </div>
                                            </td>
                                            <td class="px-6 py-4">
                                                <JobStatusChip status=job.status />
                                            </td>
                                            <td class="px-6 py-4 w-1/4">
                                                <div class="flex items-center gap-3">
                                                     <div class="flex-1 h-1 bg-gray-800 rounded-full overflow-hidden">
                                                        <div 
                                                            class="h-full bg-blue-500 transition-all duration-500" 
                                                            style=format!("width: {}%", job.progress)
                                                        ></div>
                                                     </div>
                                                     <span class="text-[10px] font-mono text-gray-400">{job.progress}"%"</span>
                                                </div>
                                            </td>
                                            <td class="px-6 py-4 text-xs text-gray-400">{job.started_at}</td>
                                            <td class="px-6 py-4 text-right">
                                                <button 
                                                    on:click=move |_| {
                                                        set_selected_job_name.set(name.clone());
                                                        set_selected_job_id.set(Some(id.clone()));
                                                    }
                                                    class="p-1.5 text-blue-400 hover:bg-blue-500/10 rounded-md transition duration-200"
                                                    title="View Logs"
                                                >
                                                    "📁"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view().into_any(),
                                Err(_) => view! { <tr><td colspan="5" class="px-6 py-8 text-center text-red-400">"Error loading job queue."</td></tr> }.into_any(),
                            })}
                        </Suspense>
                    </tbody>
                </table>
            </div>

            // Logs Modal
            {move || selected_job_id.get().map(|jid| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                    <div class="absolute inset-0 bg-black/80 backdrop-blur-sm" on:click=move |_| set_selected_job_id.set(None)></div>
                    <div class="relative bg-gray-900 border border-gray-700 rounded-2xl shadow-2xl w-full max-w-4xl max-h-[80vh] flex flex-col overflow-hidden">
                        <div class="px-6 py-4 border-b border-gray-800 flex justify-between items-center bg-gray-900">
                            <h2 class="text-sm font-bold text-white uppercase tracking-widest flex items-center gap-2">
                                <span class="w-2 h-2 rounded-full bg-blue-500 animate-pulse"></span>
                                "Logs: " {selected_job_name.get()} " (" {jid.chars().take(8).collect::<String>()} ")"
                            </h2>
                            <button on:click=move |_| set_selected_job_id.set(None) class="text-gray-500 hover:text-white text-2xl">"×"</button>
                        </div>
                        <div class="flex-1 overflow-y-auto p-6 font-mono text-[11px] bg-black/40 custom-scrollbar">
                            <Suspense fallback=|| view! { <div class="text-gray-500 italic">"Fetching runtime logs..."</div> }>
                                {move || job_logs_res.get().map(|r| {
                                    let logs = r.unwrap_or_default();
                                    if logs.is_empty() {
                                        view! { <div class="text-gray-600 italic">"No logs available for this job."</div> }.into_any()
                                    } else {
                                        logs.into_iter().map(|log| view! {
                                            <div class="flex gap-4 mb-1 group">
                                                <span class="text-gray-600 shrink-0">"[" {log.timestamp} "]"</span>
                                                <span class="text-blue-300 group-hover:text-blue-200 transition-colors whitespace-pre-wrap">{log.message}</span>
                                            </div>
                                        }).collect_view().into_any()
                                    }
                                })}
                            </Suspense>
                        </div>
                        <div class="px-6 py-3 border-t border-gray-800 bg-gray-900/50 flex justify-between items-center">
                            <span class="text-[10px] text-gray-500 italic">"Logs are polled automatically every 2 seconds"</span>
                            <button on:click=move |_| set_selected_job_id.set(None) class="px-4 py-1.5 bg-gray-800 hover:bg-gray-700 text-white rounded text-xs font-semibold">"Close"</button>
                        </div>
                    </div>
                </div>
            })}
        </div>
    }
}

#[component]
fn JobStatusChip(status: String) -> impl IntoView {
    let classes = match status.as_str() {
        "pending" => "bg-gray-500/10 text-gray-400 border-gray-500/20",
        "running" => "bg-blue-500/10 text-blue-400 border-blue-500/20 animate-pulse",
        "completed" => "bg-green-500/10 text-green-400 border-green-500/20",
        "failed" => "bg-red-500/10 text-red-400 border-red-500/20 font-bold",
        _ => "bg-gray-500/10 text-gray-400 border-gray-500/20",
    };
    
    view! {
        <span class=format!("text-[10px] font-bold px-2 py-0.5 rounded-full border uppercase tracking-wider {}", classes)>
            {status}
        </span>
    }
}
