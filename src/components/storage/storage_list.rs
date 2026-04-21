use leptos::prelude::*;
use crate::storage::*;

#[component]
pub fn StoragePage() -> impl IntoView {
    let (show_add_modal, set_show_add_modal) = signal(false);
    let (f_name, set_f_name) = signal(String::new());
    let (f_size, set_f_size) = signal(10i32);
    let (f_replicas, set_f_replicas) = signal(2i32);
    let (last_job_id, _set_last_job_id) = signal(None::<String>);
    let (browsing_volume, set_browsing_volume) = signal(Option::<String>::None);

    let list_volumes_res = Resource::new(|| (), |_| async { list_volumes().await });
    let status_res = Resource::new(|| (), |_| async { get_storage_status().await });
    let create_action = ServerAction::<CreateVolume>::new();
    let delete_action = ServerAction::<DeleteVolume>::new();
    let initialize_action = ServerAction::<InitializeVolume>::new();
    let (initializing_volume, set_initializing_volume) = signal(Option::<String>::None);

    // Polling for logs
    let (poll_trigger, _set_poll_trigger) = signal(0);
    #[cfg(target_arch = "wasm32")]
    let _ = {
        use gloo_timers::callback::Interval;
        let interval = Interval::new(2000, move || {
            _set_poll_trigger.update(|n| *n += 1);
        });
        interval
    };

    let job_logs_res = Resource::new(move || (last_job_id.get(), poll_trigger.get()), |(jid, _)| async move {
        if let Some(id) = jid {
            crate::jobs::list_job_logs(id).await
        } else {
            Ok(vec![])
        }
    });

    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        if let Some(Ok(job_id)) = create_action.value().get() {
            _set_last_job_id.set(Some(job_id));
            set_show_add_modal.set(false);
            set_f_name.set(String::new());
        }
    });

    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        if let Some(Ok(logs)) = job_logs_res.get() {
            if logs.iter().any(|l| l.message.to_lowercase().contains("complete")) {
                list_volumes_res.refetch();
            }
        }
    });

    let volumes = move || list_volumes_res.get().map(|r| r.unwrap_or_default()).unwrap_or_default();
    let status = move || status_res.get().and_then(|r| r.ok());

    let input_cls = "w-full px-3 py-2 bg-gray-800/80 border border-gray-700 rounded-lg text-gray-200 text-sm focus:ring-1 focus:ring-blue-500 transition-all";
    let label_cls = "block text-xs font-semibold text-gray-400 uppercase tracking-wide mb-1.5";

    view! {
        <div class="flex flex-col gap-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-white tracking-tight">"Block Storage"</h1>
                    <p class="text-sm text-gray-500 mt-1">"Manage distributed Linstor volumes and ZFS filesystems"</p>
                </div>
                <button
                    on:click=move |_| set_show_add_modal.set(true)
                    class="flex items-center gap-2 px-4 py-2.5 bg-blue-600 hover:bg-blue-500 text-white text-sm font-semibold rounded-lg transition-colors shadow-lg shadow-blue-500/20 cursor-pointer"
                >
                    <span class="text-base leading-none">"+"</span>
                    "Create Volume"
                </button>
            </div>

            <Suspense fallback=|| view! { 
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4 animate-pulse">
                    <div class="bg-gray-900 border border-gray-800 p-5 rounded-xl h-24"></div>
                    <div class="bg-gray-900 border border-gray-800 p-5 rounded-xl h-24"></div>
                    <div class="bg-gray-900 border border-gray-800 p-5 rounded-xl h-24"></div>
                </div>
            }>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div class="bg-gray-900 border border-gray-800 p-5 rounded-xl">
                        <p class="text-xs text-gray-500 uppercase tracking-widest font-semibold mb-2">"Cluster Health"</p>
                        <div class="flex items-center gap-2">
                            <span class="w-3 h-3 bg-green-500 rounded-full animate-pulse"></span>
                            <span class="text-2xl font-bold text-green-400">"Healthy"</span>
                        </div>
                    </div>
                    <div class="bg-gray-900 border border-gray-800 p-5 rounded-xl">
                        <p class="text-xs text-gray-500 uppercase tracking-widest font-semibold mb-2">"Total Capacity"</p>
                        <span class="text-2xl font-bold text-white">
                            {move || match status() {
                                Some(s) => format!("{} GB", s.total_gb),
                                None => "0 GB".into()
                            }}
                        </span>
                    </div>
                    <div class="bg-gray-900 border border-gray-800 p-5 rounded-xl">
                        <p class="text-xs text-gray-500 uppercase tracking-widest font-semibold mb-2">"Active Volumes"</p>
                        <span class="text-2xl font-bold text-blue-400">{move || volumes().len().to_string()}</span>
                    </div>
                </div>

                <div class="bg-gray-900/60 border border-gray-800 rounded-xl overflow-hidden shadow-xl">
                    <Show 
                        when=move || !volumes().is_empty()
                        fallback=|| view! {
                            <div class="flex flex-col items-center justify-center py-20 gap-4">
                                <div class="w-16 h-16 rounded-2xl bg-gray-800 flex items-center justify-center text-3xl">"💾"</div>
                                <p class="text-gray-400">"No storage volumes found"</p>
                            </div>
                        }
                    >
                        <table class="w-full text-left">
                            <thead>
                                <tr class="border-b border-gray-800 bg-gray-900/80 text-xs text-gray-400 uppercase font-bold tracking-wider">
                                    <th class="px-6 py-4">"Name"</th>
                                    <th class="px-6 py-4">"Size"</th>
                                    <th class="px-6 py-4">"Status"</th>
                                    <th class="px-6 py-4">"Access Gateway"</th>
                                    <th class="px-6 py-4 text-right">"Actions"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || volumes().into_iter().map(|vol| {
                                    let vol_id = vol.id.clone();
                                    let s_dot = vol.status.to_lowercase();
                                    let s_text = vol.status.clone();
                                    let s_color = if s_dot == "uptodate" || s_dot == "insync" || s_dot == "online" { "bg-green-500" } else { "bg-yellow-500" };
                                    
                                    view! {
                                        <tr class="border-b border-gray-800/50 hover:bg-gray-800/30 transition-colors">
                                            <td class="px-6 py-4 font-semibold text-gray-100">{vol.name}</td>
                                            <td class="px-6 py-4 text-sm text-gray-300"><span>{vol.size_gb}</span> " GB"</td>
                                            <td class="px-6 py-4">
                                                <div class="flex flex-col gap-1">
                                                    <div class="flex items-center gap-2">
                                                        <span class=format!("w-2 h-2 rounded-full {}", if vol.last_error.is_some() { "bg-red-500" } else { s_color })></span>
                                                        <span class="text-xs text-gray-400 capitalize">{s_text}</span>
                                                    </div>
                                                    {vol.last_error.as_ref().map(|err| view! {
                                                        <div class="text-[10px] text-red-400/80 max-w-[150px] leading-tight flex items-start gap-1">
                                                            <span>"⚠️"</span>
                                                            <span>{err.clone()}</span>
                                                        </div>
                                                    })}
                                                </div>
                                            </td>
                                            <td class="px-6 py-4">
                                                <div class="flex flex-wrap gap-2">
                                                    {vol.services.into_iter().map(|srv| view! {
                                                        <span class="px-2 py-0.5 bg-gray-800 rounded text-[10px] font-mono text-gray-400 uppercase tracking-tighter">{srv}</span>
                                                    }).collect_view()}
                                                </div>
                                            </td>
                                            <td class="px-6 py-4 text-right">
                                                <div class="flex items-center justify-end gap-2">
                                                    <button 
                                                        on:click={
                                                            let vid = vol_id.clone();
                                                            move |_| set_initializing_volume.set(Some(vid.clone()))
                                                        }
                                                        class="p-2 text-gray-500 hover:text-orange-400 hover:bg-orange-400/10 rounded-lg transition-all"
                                                        title="Initialize Volume"
                                                    >
                                                        <crate::components::icons::Wrench class="w-4 h-4"/>
                                                    </button>
                                                    <button 
                                                        on:click={
                                                            let vid = vol_id.clone();
                                                            move |_| set_browsing_volume.set(Some(vid.clone()))
                                                        }
                                                        class="p-2 text-gray-500 hover:text-blue-400 hover:bg-blue-400/10 rounded-lg transition-all"
                                                        title="Browse Files"
                                                    >
                                                        <crate::components::icons::Folder class="w-4 h-4"/>
                                                    </button>
                                                    <button 
                                                        on:click=move |_| {
                                                            delete_action.dispatch(DeleteVolume { id: vol_id.clone() });
                                                        }
                                                        class="p-2 text-gray-500 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-all"
                                                        title="Delete Volume"
                                                    >
                                                        <crate::components::icons::Trash class="w-4 h-4"/>
                                                    </button>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                             </tbody>
                         </table>
                    </Show>
                </div>

            <div class="mt-4">
                <details class="bg-gray-900/40 border border-gray-800 rounded-xl overflow-hidden group">
                    <summary class="px-6 py-4 cursor-pointer hover:bg-gray-800/20 transition-all flex items-center justify-between list-none">
                        <div class="flex items-center gap-2">
                             <div class="w-2 h-2 rounded-full bg-blue-500 animate-pulse"></div>
                             <span class="text-xs font-bold text-gray-400 uppercase tracking-widest">"System Diagnosis"</span>
                        </div>
                        <span class="text-[10px] text-blue-400 uppercase font-bold">"View Raw Cluster Status"</span>
                    </summary>
                    <div class="px-6 py-6 border-t border-gray-800 bg-black/40">
                        <div class="grid grid-cols-1 gap-6">
                            <div>
                                <h3 class="text-[10px] font-bold text-gray-500 uppercase tracking-widest mb-3">"Backend Logs & API Diagnostics"</h3>
                                <pre class="text-[10px] font-mono text-gray-400 whitespace-pre-wrap p-4 bg-gray-950 rounded-lg border border-gray-800">
                                    {move || match (status_res.get(), status()) {
                                        (None, _) => "Loading system diagnostics... (Request may take up to 10s)".into(),
                                        (Some(Err(e)), _) => format!("SERVER ERROR: {}\n\nPossible cause: Linstor controller is unreachable or host address is incorrect.", e),
                                        (Some(Ok(_)), Some(s)) => s.diagnosis_output,
                                        _ => "No diagnostic data available. Please re-run host setup.".into()
                                    }}
                                </pre>
                            </div>
                        </div>
                    </div>
                </details>
            </div>
            </Suspense>

            {move || show_add_modal.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                    <div class="absolute inset-0 bg-black/70 backdrop-blur-sm" on:click=move |_| set_show_add_modal.set(false)></div>
                    <div class="relative bg-gray-900 border border-gray-700 rounded-2xl shadow-2xl w-full max-w-md p-6">
                        <h2 class="text-lg font-bold text-white mb-6">"Create Block Volume"</h2>
                        
                        {move || create_action.value().get().map(|v| match v {
                            Err(e) => view! {
                                <div class="mb-4 p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 text-xs">
                                    "Error: " {e.to_string()}
                                </div>
                            }.into_any(),
                            _ => view! {}.into_any()
                        })}

                        <div class="space-y-4">
                            <div>
                                <label class=label_cls>"Volume Name"</label>
                                <input type="text" class=input_cls placeholder="my-storage-vol" 
                                    on:input=move |ev| set_f_name.set(event_target_value(&ev)) />
                            </div>
                            <div class="grid grid-cols-2 gap-4">
                                <div>
                                    <label class=label_cls>"Size (GB)"</label>
                                    <input type="number" class=input_cls value="10" 
                                        on:input=move |ev| if let Ok(n) = event_target_value(&ev).parse() { set_f_size.set(n) } />
                                </div>
                                <div>
                                    <label class=label_cls>"Replicas"</label>
                                    <input type="number" class=input_cls value="2" 
                                        on:input=move |ev| if let Ok(n) = event_target_value(&ev).parse() { set_f_replicas.set(n) } />
                                </div>
                            </div>
                            
                            {move || last_job_id.get().map(|_| view! {
                                <div class="p-3 bg-gray-950 border border-gray-800 rounded-lg">
                                    <p class="text-[10px] text-gray-500 uppercase font-bold mb-2 tracking-widest">"System Logs"</p>
                                    <div class="space-y-1 max-h-32 overflow-y-auto font-mono text-[10px]">
                                        <Suspense fallback=|| view! { <div class="text-gray-600 italic">"Fetching logs..."</div> }>
                                            {move || job_logs_res.get().map(|r| r.unwrap_or_default().into_iter().map(|log| view! {
                                                <div class="flex gap-2">
                                                    <span class="text-gray-600">"[" {log.timestamp} "]"</span>
                                                    <span class="text-blue-400">{log.message}</span>
                                                </div>
                                            }).collect_view())}
                                        </Suspense>
                                    </div>
                                </div>
                            })}

                            <div class="pt-4 flex justify-end gap-3">
                                <button on:click=move |_| set_show_add_modal.set(false) class="px-4 py-2 text-sm text-gray-400">"Cancel"</button>
                                <button on:click=move |_| {
                                    create_action.dispatch(CreateVolume { 
                                        name: f_name.get_untracked(), 
                                        size_gb: f_size.get_untracked(),
                                        replicas: f_replicas.get_untracked()
                                    });
                                } 
                                disabled=move || create_action.pending().get()
                                class="px-6 py-2 bg-blue-600 rounded-lg font-bold text-sm disabled:opacity-50">
                                    {move || if create_action.pending().get() { "Creating..." } else { "Create" }}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            })}

            // ── File Browser Modal ──
            {move || browsing_volume.get().map(|vid| {
                let vid_display = vid.clone();
                let vid_browser = vid.clone();
                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center p-8">
                        <div class="absolute inset-0 bg-black/80 backdrop-blur-md" on:click=move |_| set_browsing_volume.set(None)></div>
                        <div class="bg-gray-900 border border-gray-800 rounded-2xl w-full max-w-6xl h-[85vh] flex flex-col relative z-20 shadow-2xl overflow-hidden scale-in-center animate-in duration-300">
                            <div class="p-4 border-b border-gray-800 bg-gray-900 flex items-center justify-between">
                                <div class="flex items-center gap-3">
                                    <div class="w-8 h-8 rounded-lg bg-blue-600/20 flex items-center justify-center">
                                        <crate::components::icons::Folder class="w-5 h-5 text-blue-400"/>
                                    </div>
                                    <div>
                                        <h2 class="text-sm font-bold text-white tracking-tight">"File Browser"</h2>
                                        <p class="text-[10px] font-mono text-gray-500 uppercase tracking-widest">{move || vid_display.clone()}</p>
                                    </div>
                                </div>
                                <button 
                                    on:click=move |_| set_browsing_volume.set(None)
                                    class="p-2 text-gray-500 hover:text-white hover:bg-gray-800 rounded-lg transition-all"
                                >
                                    <crate::components::icons::X class="w-5 h-5"/>
                                </button>
                            </div>
                            <div class="flex-1 overflow-hidden p-6">
                                <crate::components::storage::file_browser::FileBrowser volume_id=vid_browser />
                            </div>
                        </div>
                    </div>
                }
            })}

            // ── Initialize Modal ──
            {move || initializing_volume.get().map(|vid| {
                let vid_init = vid.clone();
                let vid_display = vid.clone();
                view! {
                    <div class="fixed inset-0 z-[60] flex items-center justify-center p-4">
                        <div class="absolute inset-0 bg-black/80 backdrop-blur-md" on:click=move |_| if !initialize_action.pending().get() { set_initializing_volume.set(None) }></div>
                        <div class="relative bg-gray-900 border border-gray-700 rounded-2xl shadow-2xl w-full max-w-lg p-6 overflow-hidden">
                            <div class="flex items-center gap-4 mb-6">
                                <div class="w-12 h-12 rounded-xl bg-orange-500/20 flex items-center justify-center">
                                    <crate::components::icons::Wrench class="w-6 h-6 text-orange-400"/>
                                </div>
                                <div>
                                    <h2 class="text-lg font-bold text-white">"Initialize Volume"</h2>
                                    <p class="text-xs text-gray-400 font-mono">{vid_display}</p>
                                </div>
                            </div>

                            <div class="space-y-4">
                                <div class="p-4 bg-orange-500/10 border border-orange-500/20 rounded-xl">
                                    <div class="flex gap-3">
                                        <span class="text-orange-400">"⚠️"</span>
                                        <p class="text-xs text-orange-200/80 leading-relaxed">
                                            "This will prepare the volume for use. "
                                            <span class="font-bold text-white">"Existing files are safe"</span>
                                            ": the system automatically detects existing filesystems and only formats blank volumes."
                                        </p>
                                    </div>
                                </div>

                                {move || initialize_action.value().get().map(|v| match v {
                                    Err(e) => view! {
                                        <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 text-xs font-mono">
                                            <p class="font-bold mb-1">"STDOUT/STDERR:"</p>
                                            {e.to_string()}
                                        </div>
                                    }.into_any(),
                                    Ok(_) => view! {
                                        <div class="p-3 bg-green-500/10 border border-green-500/20 rounded-lg text-green-400 text-xs flex items-center gap-2">
                                            <crate::components::icons::CheckCircle class="w-4 h-4"/>
                                            <span>"Volume successfully initialized and mounted!"</span>
                                        </div>
                                    }.into_any(),
                                })}

                                <div class="pt-4 flex justify-end gap-3">
                                    <button 
                                        on:click=move |_| set_initializing_volume.set(None) 
                                        disabled=move || initialize_action.pending().get()
                                        class="px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors"
                                    >
                                        {move || if initialize_action.value().get().is_some() { "Close" } else { "Cancel" }}
                                    </button>
                                    <Show when=move || initialize_action.value().get().is_none() && !initialize_action.pending().get()>
                                        <button 
                                            on:click={
                                                let vid = vid_init.clone();
                                                move |_| { initialize_action.dispatch(InitializeVolume { volume_id: vid.clone() }); }
                                            }
                                            class="px-6 py-2 bg-orange-600 hover:bg-orange-500 text-white rounded-lg font-bold text-sm shadow-lg shadow-orange-500/20 transition-all"
                                        >
                                            "Confirm & Initialize"
                                        </button>
                                    </Show>
                                    <Show when=move || initialize_action.pending().get()>
                                        <div class="flex items-center gap-3 px-4 text-orange-400">
                                            <div class="w-4 h-4 border-2 border-orange-400/20 border-t-orange-400 rounded-full animate-spin"></div>
                                            <span class="text-xs font-bold uppercase tracking-widest">"Orchestrating..."</span>
                                        </div>
                                    </Show>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
