use leptos::prelude::*;
#[cfg(not(feature = "ssr"))]
use leptos::task::spawn_local;
use crate::llms::*;
use crate::hosts::list_hosts;
use crate::components::icons;

#[component]
pub fn LLMsPage() -> impl IntoView {
    let (show_add_modal, set_show_add_modal) = signal(false);
    let (hf_query, set_hf_query) = signal(String::new());
    
    let (active_tab, set_active_tab) = signal("models");
    let selected_chat_model = RwSignal::new(Option::<String>::None);

    let create_action = ServerAction::<CreateLLM>::new();
    let delete_action = ServerAction::<DeleteLLM>::new();
    let status_action = ServerAction::<ToggleLLMStatus>::new();
    let sync_action = ServerAction::<SyncHostModels>::new();
    let deploy_fox_action = ServerAction::<DeployFox>::new();

    let hf_search = Resource::new(move || hf_query.get(), |q: String| async move {
        if q.is_empty() { Ok(Vec::new()) }
        else { search_hf_models(q).await }
    });

    // Resource — refetches whenever any action version changes
    let (tick, set_tick) = signal(0);
    let llms_res = Resource::new(
        move || (
            create_action.version().get(),
            delete_action.version().get(),
            status_action.version().get(),
            sync_action.version().get(),
            deploy_fox_action.version().get(),
            tick.get(),
        ),
        |_| async { list_llms().await },
    );

    // Polling while downloading
    Effect::new(move |_| {
        #[cfg(not(feature = "ssr"))]
        {
            let is_downloading = match llms_res.get() {
                Some(Ok(llms)) => llms.iter().any(|l| l.download_status.starts_with("downloading")),
                _ => false,
            };

            if is_downloading {
                spawn_local(async move {
                    gloo_timers::future::TimeoutFuture::new(2000).await;
                    set_tick.update(|t| *t += 1);
                });
            }
        }
        #[cfg(feature = "ssr")]
        {
            let _ = set_tick;
        }
    });

    // Fetch hosts for selection
    let hosts_res = Resource::new(|| (), |_| async { list_hosts().await });

    view! {
        <div class="p-6 max-w-7xl mx-auto space-y-8 animate-in fade-in duration-500">
            <div class="flex justify-between items-center text-left">
                <div>
                    <h1 class="text-3xl font-bold text-white flex items-center gap-3">
                        <icons::Brain class="w-8 h-8 text-blue-500".to_string() />
                        "LLM Management"
                    </h1>
                    <p class="text-gray-400 mt-1">"Deploy and manage large language models in host-native mode"</p>
                </div>
                <div class="flex gap-3">
                    <button 
                        on:click=move |_| set_show_add_modal.set(true)
                        class="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white font-bold rounded-xl flex items-center gap-2 transition-all shadow-lg shadow-blue-500/20"
                    >
                        <icons::Plus class="w-5 h-5".to_string() />
                        "Add Model"
                    </button>
                </div>
            </div>

            // ── Tabs Header ──────────────────────────────────────────────────
            <div class="flex gap-4 border-b border-gray-800 pb-2">
                <button 
                    on:click=move |_| set_active_tab.set("models")
                    class=move || format!("px-4 py-2 font-bold uppercase tracking-widest text-xs transition-colors rounded-t-lg {}", if active_tab.get() == "models" { "text-blue-400 bg-blue-500/10 border-b-2 border-blue-500" } else { "text-gray-500 hover:text-gray-300" })
                >
                    "Models"
                </button>
                <button 
                    on:click=move |_| set_active_tab.set("chat")
                    class=move || format!("px-4 py-2 font-bold uppercase tracking-widest text-xs transition-colors rounded-t-lg {}", if active_tab.get() == "chat" { "text-blue-400 bg-blue-500/10 border-b-2 border-blue-500" } else { "text-gray-500 hover:text-gray-300" })
                >
                    "Chat"
                </button>
            </div>

            // ── Tab Contents ─────────────────────────────────────────────────
            <div class=move || if active_tab.get() == "chat" { "block h-full" } else { "hidden" }>
                <crate::components::chat::ChatInterface selected_id=selected_chat_model />
            </div>

            <div class=move || if active_tab.get() == "models" { "block space-y-6" } else { "hidden" }>
                // ── Models Grid ──────────────────────────────────────────────────
                <div class="grid grid-cols-1 gap-6">
                <Suspense fallback=move || view! { <div class="flex justify-center p-12"><span class="loading loading-spinner loading-lg text-blue-500"></span></div> }>
                    {move || llms_res.get().map(|res| match res {
                        Ok(llms) => if llms.is_empty() {
                            view! {
                                <div class="flex flex-col items-center justify-center py-24 bg-gray-900/40 rounded-3xl border border-gray-800 border-dashed">
                                    <icons::WarningCircle class="w-16 h-16 text-gray-700 mb-4".to_string() />
                                    <p class="text-gray-400 font-medium">"No models deployed yet"</p>
                                    <p class="text-sm text-gray-500 mt-1">"Click 'Add Model' or deploy the Fox Inference Engine on a host."</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                                    {llms.into_iter().map(|llm| {
                                        let id = llm.id.clone();
                                        let h_id = llm.host_id.clone();
                                        let name = llm.name.clone();
                                        let m_name = llm.model_name.clone();
                                        let status = llm.status.clone();
                                        let h_name = llm.host_name.clone();
                                        let provider = llm.provider.clone();
                                        let dl_status = llm.download_status.clone();
                                        
                                        let status_badge_cls = if status == "online" { 
                                            "px-2 py-0.5 rounded text-[10px] font-bold uppercase border bg-green-500/10 text-green-400 border-green-500/20 cursor-pointer hover:opacity-80 transition-opacity" 
                                        } else { 
                                            "px-2 py-0.5 rounded text-[10px] font-bold uppercase border bg-amber-500/10 text-amber-400 border-amber-500/20 cursor-pointer hover:opacity-80 transition-opacity" 
                                        };
                                        
                                        view! {
                                            <div class="group relative bg-gray-900 border border-gray-800 rounded-2xl p-6 hover:border-blue-500/50 transition-all duration-300 shadow-xl overflow-hidden text-left">
                                                <div class="absolute inset-0 bg-gradient-to-br from-blue-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />
                                                
                                                <div class="relative z-10">
                                                    <div class="flex justify-between items-start mb-6">
                                                        <div class="flex items-center gap-3">
                                                            <div class="bg-blue-500/10 p-2.5 rounded-xl">
                                                                <icons::Cpu class="w-5 h-5 text-blue-500".to_string() />
                                                            </div>
                                                            <div>
                                                                <h2 class="font-bold text-white tracking-tight">{name}</h2>
                                                                <p class="text-[10px] text-gray-500 font-mono truncate max-w-[150px]">{m_name}</p>
                                                            </div>
                                                        </div>
                                                        <button 
                                                            on:click={
                                                                let id = id.clone();
                                                                move |_| { status_action.dispatch(ToggleLLMStatus { id: id.clone() }); }
                                                            }
                                                            class=status_badge_cls
                                                        >
                                                            {status}
                                                        </button>
                                                    </div>
                                                    
                                                    <div class="space-y-3 mb-8">
                                                        <div class="flex items-center justify-between text-xs text-gray-400">
                                                            <div class="flex items-center gap-2">
                                                                <icons::Desktop class="w-4 h-4 opacity-50".to_string() />
                                                                <span>"Host"</span>
                                                            </div>
                                                            <span class="font-semibold text-gray-300">{h_name}</span>
                                                        </div>
                                                        <div class="flex items-center justify-between text-xs text-gray-400">
                                                            <div class="flex items-center gap-2">
                                                                <icons::Cube class="w-4 h-4 opacity-50".to_string() />
                                                                <span>"Provider"</span>
                                                            </div>
                                                            <span class="px-1.5 py-0.5 bg-gray-800 rounded text-blue-400 font-bold uppercase tracking-tighter">{provider}</span>
                                                        </div>
                                                        {(!dl_status.is_empty() && dl_status != "none" && dl_status != "complete").then(move || {
                                                            let dl_text = dl_status.clone();
                                                            view! {
                                                                <div class="mt-4 p-3 bg-blue-500/10 border border-blue-500/20 rounded-xl">
                                                                    <div class="flex justify-between items-center text-[10px] mb-1">
                                                                        <span class="text-blue-300 font-bold uppercase">"Downloading"</span>
                                                                        <span class="text-blue-300">{dl_text}</span>
                                                                    </div>
                                                                    <div class="h-1 bg-gray-800 rounded-full overflow-hidden">
                                                                        <div class="h-full bg-blue-500 animate-pulse w-full" />
                                                                    </div>
                                                                </div>
                                                            }
                                                        })}
                                                    </div>

                                                    <div class="flex gap-2 pt-4 border-t border-gray-800/50">
                                                        <button 
                                                            on:click={
                                                                let h_id = h_id.clone();
                                                                move |_| { sync_action.dispatch(SyncHostModels { host_id: h_id.clone() }); }
                                                            }
                                                            class="flex items-center gap-1.5 text-[10px] font-bold uppercase text-gray-500 hover:text-white transition-colors"
                                                            title="Sync with Host"
                                                        >
                                                            {move || {
                                                                let cls = format!("w-3.5 h-3.5 {}", if sync_action.pending().get() { "animate-spin" } else { "" });
                                                                view! { <icons::ArrowCounterClockwise class=cls /> }
                                                            }}
                                                            "Sync"
                                                        </button>
                                                        <div class="flex-1" />
                                                        <button 
                                                            on:click={
                                                                let id = id.clone();
                                                                move |_| {
                                                                    selected_chat_model.set(Some(id.clone()));
                                                                    set_active_tab.set("chat");
                                                                }
                                                            }
                                                            class="p-1 text-blue-400 hover:text-blue-300 transition-colors"
                                                            title="Chat with Model"
                                                        >
                                                            <lepticons::Icon glyph=lepticons::LucideGlyph::MessageSquare class="w-4 h-4" />
                                                        </button>
                                                        <button 
                                                            on:click={
                                                                let id = id.clone();
                                                                move |_| { delete_action.dispatch(DeleteLLM { id: id.clone() }); }
                                                            }
                                                            class="p-1 text-gray-600 hover:text-red-400 transition-colors"
                                                        >
                                                            <icons::Trash class="w-4 h-4".to_string() />
                                                        </button>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        },
                        Err(e) => view! { 
                            <div class="p-6 bg-red-500/10 border border-red-500/20 rounded-2xl flex items-center gap-4 text-red-400">
                                <icons::WarningCircle class="w-6 h-6".to_string() />
                                <span>{format!("Error loading models: {}", e)}</span>
                            </div>
                        }.into_any(),
                    })}
                </Suspense>
            </div>

            // ── Deployment Quick Actions ─────────────────────────────────────
            <div class="bg-gray-900 border border-gray-800 rounded-3xl p-8">
                <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-6">
                    <div class="space-y-1 text-left">
                        <h2 class="text-xl font-bold text-white flex items-center gap-2">
                            <icons::Play class="w-6 h-6 text-indigo-400".to_string() />
                            "Deploy Inferencing Engine"
                        </h2>
                        <p class="text-sm text-gray-500">"Deploy the Fox Inference Engine in host-native mode for maximum performance."</p>
                    </div>
                    <Suspense fallback=|| view! { <div class="flex gap-2">{(0..2).map(|_| view!{<div class="w-32 h-10 bg-gray-800 animate-pulse rounded-xl"></div>}).collect_view()}</div> }>
                        {move || hosts_res.get().map(|res| match res {
                            Ok(hosts) => view! {
                                <div class="flex flex-wrap gap-2">
                                    {hosts.into_iter().filter(|h| h.status == "online").map(|h| {
                                        let h_id_btn = h.id.clone();
                                        let h_name = h.name.clone();
                                        view! {
                                            <button 
                                                on:click={
                                                    let h_id_dispatch = h_id_btn.clone();
                                                    move |_| { deploy_fox_action.dispatch(DeployFox { host_id: h_id_dispatch.clone() }); }
                                                }
                                                disabled=deploy_fox_action.pending()
                                                class="px-4 py-2 bg-indigo-500/10 hover:bg-indigo-500 border border-indigo-500/20 text-indigo-400 hover:text-white font-bold text-xs rounded-xl flex items-center gap-2 transition-all disabled:opacity-50"
                                            >
                                                {move || {
                                                    let cls = format!("w-3.5 h-3.5 {}", if deploy_fox_action.pending().get() { "animate-pulse" } else { "" });
                                                    view! { <icons::Wrench class=cls /> }
                                                }}
                                                {format!("Deploy on {}", h_name)}
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any(),
                            _ => view! { <div class="hidden" /> }.into_any()
                        })}
                    </Suspense>
                </div>
            </div>
            </div>

            // ── Add Model Modal ──────────────────────────────────────────────
            {move || show_add_modal.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                    <div class="absolute inset-0 bg-black/70 backdrop-blur-sm" on:click=move |_| set_show_add_modal.set(false) />
                    <div class="relative bg-gray-900 border border-gray-700 rounded-2xl shadow-2xl w-full max-w-2xl overflow-hidden flex flex-col h-[600px]">
                        <div class="flex items-center justify-between px-6 py-5 border-b border-gray-800 bg-black/20">
                            <h2 class="text-base font-bold text-white flex items-center gap-2">
                                <icons::Plus class="w-5 h-5 text-blue-400".to_string() />
                                "Deploy HuggingFace Model"
                            </h2>
                            <button on:click=move |_| set_show_add_modal.set(false) class="text-gray-500 hover:text-white transition-colors">"×"</button>
                        </div>
                        
                        <div class="p-6 border-b border-gray-800">
                            <input 
                                type="text"
                                placeholder="Search HuggingFace (e.g. meta-llama/Llama-2-7b-chat-hf)..."
                                class="w-full bg-black/40 border border-gray-700 rounded-xl px-4 py-3 text-white focus:outline-none focus:border-blue-500 text-sm"
                                on:input=move |ev| set_hf_query.set(event_target_value(&ev))
                                prop:value=hf_query
                            />
                        </div>

                        <div class="flex-1 overflow-y-auto p-6 space-y-3 custom-scrollbar">
                            <Suspense fallback=|| view! { <div class="text-center p-8 text-gray-500 text-sm">"Searching repository..."</div> }>
                                {move || hf_search.get().map(|res| match res {
                                    Ok(models) => if models.is_empty() {
                                        view! { <div class="text-center p-8 text-gray-600 text-sm italic">"Start typing to search models..."</div> }.into_any()
                                    } else {
                                        models.into_iter().map(|m| {
                                            let (target_host, set_target_host) = signal(String::new());
                                            
                                            // Pre-select first host if available
                                            Effect::new(move |_| {
                                                if let Some(Ok(hosts)) = hosts_res.get() {
                                                    if let Some(first) = hosts.iter().filter(|h| h.status == "online").next() {
                                                        if target_host.get().is_empty() {
                                                            set_target_host.set(first.id.clone());
                                                        }
                                                    }
                                                }
                                            });

                                            view! {
                                                <div class="flex items-center justify-between p-4 bg-gray-800/50 border border-gray-700 rounded-xl hover:border-gray-500 transition-colors">
                                                    <div class="text-left">
                                                        <h3 class="text-sm font-bold text-white mb-1">{m.id.clone()}</h3>
                                                        <div class="flex items-center gap-3 text-[10px] text-gray-500 font-mono">
                                                            <span>{format!("{} Downloads", m.downloads.unwrap_or(0))}</span>
                                                            <span>{format!("{} Likes", m.likes.unwrap_or(0))}</span>
                                                        </div>
                                                    </div>
                                                    <div class="flex items-center gap-2">
                                                        <select 
                                                            class="bg-black/60 border border-gray-600 rounded text-xs text-white px-2 py-1"
                                                            on:change=move |ev| set_target_host.set(event_target_value(&ev))
                                                            prop:value=target_host
                                                        >
                                                            <Suspense fallback=|| view!{ <option>"Loading..."</option> }>
                                                            {move || hosts_res.get().map(|r| r.unwrap_or_default().into_iter().filter(|h| h.status == "online").map(|h| view! { <option value=h.id>{h.name}</option> }).collect_view())}
                                                            </Suspense>
                                                        </select>
                                                        <button 
                                                            on:click={
                                                                let m_id = m.id.clone();
                                                                move |_| {
                                                                    let host_id = target_host.get();
                                                                    if host_id.is_empty() { return; }
                                                                    
                                                                    let parts: Vec<&str> = m_id.split('/').collect();
                                                                    let provider = if parts.len() > 1 { parts[0].to_string() } else { "huggingface".to_string() };
                                                                    let name = if parts.len() > 1 { parts[1].to_string() } else { m_id.clone() };
                                                                    
                                                                    create_action.dispatch(CreateLLM {
                                                                        host_id,
                                                                        name: name.clone(),
                                                                        provider,
                                                                        model_name: m_id.clone(),
                                                                    });
                                                                    set_show_add_modal.set(false);
                                                                    set_hf_query.set(String::new());
                                                                }
                                                            }
                                                            class="bg-blue-600 hover:bg-blue-500 text-white text-xs font-bold px-3 py-1.5 rounded transition-colors"
                                                        >"Deploy"</button>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view().into_any()
                                    },
                                    Err(e) => view! { <div class="text-red-400 text-sm p-4">{format!("Search failed: {}", e)}</div> }.into_any()
                                })}
                            </Suspense>
                        </div>
                    </div>
                </div>
            })}

        </div>
    }
}
