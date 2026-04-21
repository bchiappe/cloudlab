use leptos::prelude::*;
use crate::api_keys::*;
use lepticons::{Icon, LucideGlyph};

#[component]
pub fn ApiKeysManager() -> impl IntoView {
    let generate_key = ServerAction::<GenerateApiKey>::new();
    let delete_key = ServerAction::<DeleteApiKey>::new();

    let keys_resource = Resource::new(
        move || (generate_key.version().get(), delete_key.version().get()),
        |_| async move { list_api_keys().await.unwrap_or_default() }
    );

    let (new_key_label, set_new_key_label) = signal(String::new());
    let (show_key, set_show_key) = signal(Option::<String>::None);

    Effect::new(move |_| {
        if let Some(Ok(key)) = generate_key.value().get() {
            set_show_key.set(Some(key));
            set_new_key_label.set(String::new());
        }
    });

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h2 class="text-xl font-semibold text-white">"AI API Keys"</h2>
                    <p class="text-sm text-gray-400">"Manage keys for OpenAI-compatible access to your local LLMs."</p>
                </div>
            </div>

            // Generate New Key
            <div class="p-6 bg-gray-900/50 border border-gray-800 rounded-xl">
                <div class="flex flex-col md:flex-row gap-4">
                    <div class="flex-1">
                        <label class="block text-sm font-medium text-gray-400 mb-1">"Key Label"</label>
                        <input
                            type="text"
                            placeholder="e.g. My Website, VS Code"
                            class="w-full bg-black/40 border border-gray-800 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-blue-500 transition-colors"
                            prop:value=new_key_label
                            on:input=move |ev| set_new_key_label.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="flex items-end">
                        <button
                            class="px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium transition-all flex items-center gap-2 disabled:opacity-50"
                            disabled=move || new_key_label.get().is_empty() || generate_key.pending().get()
                            on:click=move |_| { generate_key.dispatch(GenerateApiKey { label: new_key_label.get() }); }
                        >
                            <Icon glyph=LucideGlyph::Plus class="w-4 h-4"/>
                            "Generate Key"
                        </button>
                    </div>
                </div>
            </div>

            // Reveal Key Modal
            {move || show_key.get().map(|key| view! {
                <div class="p-4 bg-yellow-500/10 border border-yellow-500/20 rounded-lg mb-6">
                    <div class="flex items-center gap-3 text-yellow-500 mb-2">
                        <Icon glyph=LucideGlyph::Key class="w-5 h-5"/>
                        <span class="font-bold">"Save your key!"</span>
                    </div>
                    <p class="text-sm text-gray-300 mb-4">"For security reasons, we can only show this key once. Copy it now and store it safely."</p>
                    <div class="flex items-center gap-2 bg-black/60 p-3 rounded-lg border border-yellow-500/30">
                        <code class="flex-1 text-white font-mono">{key.clone()}</code>
                        <button 
                            class="p-2 hover:bg-white/10 rounded transition-colors"
                            on:click=move |_| {
                                let _ = window().navigator().clipboard().write_text(&key);
                                set_show_key.set(None);
                            }
                        >
                            <Icon glyph=LucideGlyph::Copy class="w-4 h-4 text-gray-400"/>
                        </button>
                    </div>
                </div>
            })}

            // Keys List
            <div class="overflow-x-auto">
                <table class="w-full text-left">
                    <thead>
                        <tr class="border-b border-gray-800 text-gray-500 text-sm">
                            <th class="px-4 py-3 font-medium">"Label"</th>
                            <th class="px-4 py-3 font-medium">"Prefix"</th>
                            <th class="px-4 py-3 font-medium">"Created"</th>
                            <th class="px-4 py-3 font-medium text-right">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-gray-800">
                        <Suspense fallback=move || view! { <tr><td colspan="4" class="p-8 text-center text-gray-500">"Loading keys..."</td></tr> }>
                            {move || keys_resource.get().map(|keys: Vec<ApiKey>| {
                                if keys.is_empty() {
                                    view! { <tr><td colspan="4" class="p-8 text-center text-gray-500">"No API keys generated yet."</td></tr> }.into_any()
                                } else {
                                    keys.into_iter().map(|key| {
                                        let key_id_for_delete = key.id.clone();
                                        view! {
                                            <tr class="group hover:bg-white/[0.02] transition-colors">
                                                <td class="px-4 py-4 text-white font-medium">{key.label}</td>
                                                <td class="px-4 py-4"><code class="text-gray-400 font-mono text-sm">{key.key_prefix}"..."</code></td>
                                                <td class="px-4 py-4 text-sm text-gray-500">{key.created_at}</td>
                                                <td class="px-4 py-4 text-right">
                                                    <button 
                                                        class="p-2 text-gray-500 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-all"
                                                        on:click=move |_| { delete_key.dispatch(DeleteApiKey { id: key_id_for_delete.clone() }); }
                                                    >
                                                        <Icon glyph=LucideGlyph::Trash2 class="w-4 h-4"/>
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view().into_any()
                                }
                            })}
                        </Suspense>
                    </tbody>
                </table>
            </div>
        </div>
    }
}
