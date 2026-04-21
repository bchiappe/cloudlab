use leptos::prelude::*;
use lepticons::{Icon, LucideGlyph};
use crate::llms::*;
use leptos::task::spawn_local;

#[component]
pub fn ChatInterface() -> impl IntoView {
    let (messages, set_messages) = signal(Vec::<ChatMessage>::new());
    let (input, set_input) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (selected_id, set_selected_id) = signal(Option::<String>::None);
    let (active_thread_id, set_active_thread_id) = signal(Option::<String>::None);
    let (show_sidebar, set_show_sidebar) = signal(true);

    let list_llms_resource = Resource::new(|| (), |_| async move { list_llms().await.unwrap_or_default() });
    
    // Resource to list threads for the selected LLM
    let list_threads_resource = Resource::new(move || selected_id.get(), |llm_id| async move {
        if let Some(id) = llm_id {
            list_chat_threads(id).await.unwrap_or_default()
        } else {
            Vec::new()
        }
    });

    // Effect to trigger thread list refresh when needed
    // In a real app we'd use a Trigger or refresh the resource manually
    
    let clear_chat = move || {
        set_messages.set(Vec::new());
        set_active_thread_id.set(None);
        set_input.set(String::new());
    };
    
    let start_new_chat = move |_| clear_chat();

    let load_thread = move |thread_id: String| {
        set_is_loading.set(true);
        set_active_thread_id.set(Some(thread_id.clone()));
        spawn_local(async move {
            match get_chat_history(thread_id).await {
                Ok(history) => set_messages.set(history),
                Err(e) => set_messages.set(vec![ChatMessage { role: "assistant".into(), content: format!("Error loading history: {}", e) }]),
            }
            set_is_loading.set(false);
        });
    };

    let send_message = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let current_input = input.get();
        if current_input.trim().is_empty() || is_loading.get() { return; }

        let llm_id = match selected_id.get() {
            Some(id) => id,
            None => return,
        };

        let tid = active_thread_id.get();

        // Add user message to UI immediately
        let new_user_msg = ChatMessage { role: "user".into(), content: current_input.clone() };
        set_messages.update(|ms| ms.push(new_user_msg));
        set_input.set(String::new());
        set_is_loading.set(true);
        
        let current_history = messages.get();
        spawn_local(async move {
            match send_chat_message(llm_id, tid.clone(), current_history).await {
                Ok((new_tid, response)) => {
                    if tid.is_none() {
                        set_active_thread_id.set(Some(new_tid));
                        list_threads_resource.refetch();
                    }
                    set_messages.update(|ms| ms.push(ChatMessage { 
                        role: "assistant".into(), 
                        content: response
                    }));
                }
                Err(e) => {
                    set_messages.update(|ms| ms.push(ChatMessage { 
                        role: "assistant".into(), 
                        content: format!("Error: {}", e)
                    }));
                }
            }
            set_is_loading.set(false);
        });
    };

    view! {
        <div class="flex h-[750px] bg-gray-950 border border-gray-800 rounded-3xl overflow-hidden shadow-3xl">
            // Sidebar
            <div class=move || format!("transition-all duration-300 border-r border-gray-800 bg-gray-900/40 flex flex-col {}", if show_sidebar.get() { "w-72" } else { "w-0 opacity-0 invisible" })>
                <div class="p-4 border-b border-gray-800 flex items-center justify-between">
                    <h3 class="text-xs font-bold text-gray-400 uppercase tracking-widest">"Conversations"</h3>
                    <button 
                        on:click=start_new_chat
                        class="p-1.5 bg-blue-600/10 text-blue-400 rounded-lg hover:bg-blue-600/20 transition-all"
                        title="New Chat"
                    >
                        <Icon glyph=LucideGlyph::Plus class="w-4 h-4"/>
                    </button>
                </div>
                
                <div class="flex-1 overflow-y-auto p-2 space-y-1 custom-scrollbar">
                    <Suspense fallback=|| view! { <div class="px-3 py-2 text-xs text-gray-500">"Loading chats..."</div> }>
                        {move || list_threads_resource.get().map(|threads| {
                            if threads.is_empty() {
                                view! { <div class="px-3 py-8 text-center text-xs text-gray-600 italic">"No previous sessions"</div> }.into_any()
                            } else {
                                threads.into_iter().map(|t| {
                                    let is_active = active_thread_id.get() == Some(t.id.clone());
                                    let thread_id = t.id.clone();
                                    view! {
                                        <button 
                                            on:click=move |_| load_thread(thread_id.clone())
                                            class=move || format!("w-full text-left px-3 py-2.5 rounded-xl text-xs transition-all group flex items-center gap-3 {}", 
                                                if is_active { "bg-blue-600/10 text-blue-400 border border-blue-600/20" } 
                                                else { "text-gray-400 hover:bg-gray-800 hover:text-gray-200" })
                                        >
                                            <Icon glyph=LucideGlyph::MessageSquare class="w-3.5 h-3.5 shrink-0"/>
                                            <span class="truncate pr-4">{t.title}</span>
                                            {if is_active { view! { <div class="w-1 h-1 rounded-full bg-blue-400 ml-auto"></div> }.into_any() } else { view! {}.into_any() }}
                                        </button>
                                    }
                                }).collect_view().into_any()
                            }
                        })}
                    </Suspense>
                </div>
            </div>

            // Main Chat Area
            <div class="flex-1 flex flex-col min-w-0">
                // Header
                <div class="px-6 py-4 bg-gray-900/50 border-b border-gray-800 flex items-center justify-between backdrop-blur-md">
                    <div class="flex items-center gap-3">
                        <button 
                            on:click=move |_| set_show_sidebar.update(|s| *s = !*s)
                            class="p-2 text-gray-500 hover:text-white transition-colors"
                        >
                            <Icon glyph=LucideGlyph::Menu class="w-5 h-5"/>
                        </button>
                        <div class="p-2 bg-blue-500/10 rounded-xl">
                            <Icon glyph=LucideGlyph::MessageCircle class="w-5 h-5 text-blue-400"/>
                        </div>
                        <div>
                            <h3 class="font-semibold text-white text-sm">"AI Assistant"</h3>
                            <div class="flex items-center gap-1.5 leading-none">
                                <span class="w-1 h-1 rounded-full bg-green-500"></span>
                                <span class="text-[9px] text-gray-500 uppercase tracking-widest font-black">"CloudLab Live"</span>
                            </div>
                        </div>
                    </div>
                    
                    <div class="flex items-center gap-4">
                        <Suspense fallback=|| view! { <div class="w-32 h-8 bg-gray-800 animate-pulse rounded"></div> }>
                            <select 
                                class="bg-black/40 border border-gray-800 rounded-xl px-4 py-2 text-xs text-white focus:outline-none focus:border-blue-500 transition-all cursor-pointer hover:bg-black/60"
                                on:change=move |ev| {
                                    set_selected_id.set(Some(event_target_value(&ev)));
                                    clear_chat();
                                }
                            >
                                <option value="">"Select Model"</option>
                                {move || list_llms_resource.get().map(|llms| {
                                    llms.into_iter()
                                        .filter(|l| l.status == "online" && l.model_name != "Fox Engine")
                                        .map(|l| view! { <option value=l.id.clone()>{format!("{} ({})", l.name, l.host_name)}</option> })
                                        .collect_view()
                                })}
                            </select>
                        </Suspense>
                        
                        <div class="h-6 w-px bg-gray-800 mx-2"></div>
                        
                        <button 
                            class="p-2 text-gray-500 hover:text-white transition-colors rounded-lg hover:bg-gray-800"
                            on:click=move |_| {
                                if let Some(tid) = active_thread_id.get() {
                                    let tid_c = tid.clone();
                                    let list_threads_resource = list_threads_resource;
                                    spawn_local(async move {
                                        let _ = delete_chat_thread(tid_c).await;
                                        clear_chat();
                                        list_threads_resource.refetch();
                                    });
                                } else {
                                    clear_chat();
                                }
                            }
                            title="Clear Chat"
                        >
                            <Icon glyph=LucideGlyph::Trash2 class="w-4 h-4"/>
                        </button>
                    </div>
                </div>

                // Messages
                <div class="flex-1 overflow-y-auto p-8 space-y-8 custom-scrollbar bg-gray-950/20">
                    {move || -> AnyView { if messages.get().is_empty() {
                        view! {
                            <div class="h-full flex flex-col items-center justify-center text-center space-y-6">
                                <div class="relative">
                                    <div class="absolute inset-0 bg-blue-500 blur-3xl opacity-10 animate-pulse"></div>
                                    <div class="relative w-24 h-24 bg-blue-500/5 rounded-full flex items-center justify-center border border-blue-500/10">
                                        <Icon glyph=LucideGlyph::Sparkles class="w-10 h-10 text-blue-400 opacity-50"/>
                                    </div>
                                </div>
                                <div class="space-y-2">
                                    <h4 class="text-white text-lg font-medium tracking-tight">"Welcome to CloudLab AI"</h4>
                                    <p class="text-sm text-gray-500 max-w-sm mx-auto leading-relaxed">"Select a deployed model from the dropdown above and start a secure, private conversation."</p>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        messages.get().into_iter().map(|msg| {
                            let is_bot = msg.role == "assistant";
                            view! {
                                <div class=move || format!("flex gap-5 {}", if is_bot { "" } else { "flex-row-reverse" })>
                                    <div class=move || format!("w-9 h-9 rounded-2xl flex items-center justify-center shrink-0 shadow-lg {}", 
                                        if is_bot { "bg-gradient-to-br from-blue-600 to-indigo-700 border border-white/10" } else { "bg-gray-800 border border-gray-700" })
                                    >
                                        {if is_bot { view! { <Icon glyph=LucideGlyph::Bot class="w-5 h-5 text-white"/> }.into_any() } else { view! { <Icon glyph=LucideGlyph::User class="w-5 h-5 text-gray-400"/> }.into_any() }}
                                    </div>
                                    <div class=move || format!("max-w-[85%] rounded-[24px] px-5 py-3.5 text-sm leading-relaxed shadow-sm {}", 
                                        if is_bot { "bg-gray-900/80 text-gray-100 rounded-tl-none border border-gray-800/80 backdrop-blur-sm shadow-inner" } 
                                        else { "bg-blue-600 text-white rounded-tr-none shadow-blue-900/20" })
                                    >
                                        {msg.content}
                                    </div>
                                </div>
                            }.into_any()
                        }).collect_view().into_any()
                    }}.into_any()}

                    {move || -> Option<AnyView> { is_loading.get().then(|| view! {
                        <div class="flex gap-5">
                            <div class="w-9 h-9 rounded-2xl bg-gradient-to-br from-blue-600 to-indigo-700 flex items-center justify-center shrink-0 border border-white/10">
                                <Icon glyph=LucideGlyph::Bot class="w-5 h-5 text-white"/>
                            </div>
                            <div class="bg-gray-900/80 border border-gray-800/80 rounded-[24px] rounded-tl-none px-6 py-4 flex gap-1.5 items-center backdrop-blur-sm">
                                <span class="w-1.5 h-1.5 bg-blue-400 rounded-full animate-bounce [animation-duration:0.6s]"></span>
                                <span class="w-1.5 h-1.5 bg-blue-500 rounded-full animate-bounce [animation-duration:0.6s] [animation-delay:0.1s]"></span>
                                <span class="w-1.5 h-1.5 bg-blue-600 rounded-full animate-bounce [animation-duration:0.6s] [animation-delay:0.2s]"></span>
                            </div>
                        </div>
                    }.into_any())}}
                </div>

                // Input Area
                <div class="p-8 bg-gray-900/30 border-t border-gray-800/50 backdrop-blur-md">
                    <form class="relative max-w-4xl mx-auto" on:submit=send_message>
                        <input
                            type="text"
                            placeholder=move || if selected_id.get().is_none() { "Please select a model above to begin..." } else { "Type your message here..." }
                            class="w-full bg-black/40 border border-gray-800 rounded-2xl pl-5 pr-14 py-4 text-white focus:outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-500/10 transition-all disabled:opacity-50 text-sm shadow-inner"
                            disabled=move || is_loading.get() || selected_id.get().is_none()
                            prop:value=input
                            on:input=move |ev| set_input.set(event_target_value(&ev))
                        />
                        <button
                            type="submit"
                            class="absolute right-2.5 top-2.5 p-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-xl transition-all disabled:opacity-50 disabled:grayscale shadow-lg shadow-blue-900/20 active:scale-95"
                            disabled=move || input.get().trim().is_empty() || is_loading.get() || selected_id.get().is_none()
                        >
                            <Icon glyph=LucideGlyph::Send class="w-4 h-4"/>
                        </button>
                    </form>
                    <div class="mt-3 text-[10px] text-center text-gray-600 font-medium uppercase tracking-[0.2em]">
                        "CloudLab End-to-End Private AI Session"
                    </div>
                </div>
            </div>
        </div>
    }
}
