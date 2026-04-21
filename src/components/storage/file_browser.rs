use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use crate::storage_ssr::FileEntry;
use crate::storage::{list_files, get_file_preview, CreateFolder, DeleteFiles, RenameFile, GetFilePreview};
use crate::components::icons::{Folder, File, Trash, Edit, Upload, Download, X, ChevronRight, Search};

#[derive(Clone, Debug, PartialEq)]
enum ClipboardOp {
    _Copy(Vec<String>),
    _Cut(Vec<String>),
}

#[component]
pub fn FileBrowser(
    volume_id: String,
    #[prop(default = "".to_string())] initial_path: String,
) -> impl IntoView {
    let vid_stored = StoredValue::new(volume_id);
    let (current_path, set_current_path) = signal(initial_path);
    let (selected_files, set_selected_files) = signal(std::collections::HashSet::<String>::new());
    let (_clipboard, _set_clipboard) = signal(Option::<ClipboardOp>::None);
    let (search_query, set_search_query) = signal(String::new());
    let (show_new_folder, set_show_new_folder) = signal(false);
    let (preview_file, set_preview_file) = signal(Option::<String>::None);
    let (uploading, set_uploading) = signal(false);
    let (drag_active, set_drag_active) = signal(false);
 
    let files_res = Resource::new(
        move || (vid_stored.get_value(), current_path.get()),
        |(vid, path)| async move {
            list_files(vid, path).await
        }
    );
 
    let create_folder_action = ServerAction::<CreateFolder>::new();
    let delete_action = ServerAction::<DeleteFiles>::new();
    let _rename_action = ServerAction::<RenameFile>::new();
    let _preview_action = ServerAction::<GetFilePreview>::new();

    // Reset selection when path changes
    Effect::new(move |_| {
        current_path.track();
        set_selected_files.set(std::collections::HashSet::new());
    });

    // Refetch files when actions complete
    Effect::new(move |_| {
        if create_folder_action.version().get() > 0 {
            files_res.refetch();
        }
    });

    Effect::new(move |_| {
        if delete_action.version().get() > 0 {
            files_res.refetch();
        }
    });

    let navigate_to = move |name: String| {
        let p = current_path.get();
        let new_p = if p.is_empty() { name } else { format!("{}/{}", p, name) };
        set_current_path.set(new_p);
    };

    let navigate_up = move |idx: usize| {
        let p = current_path.get();
        let parts: Vec<&str> = p.split('/').filter(|s| !s.is_empty()).collect();
        let new_p = parts[..idx+1].join("/");
        set_current_path.set(new_p);
    };

    let root_nav = move |_| set_current_path.set("".to_string());

    let is_text_file = |name: &str| {
        let n = name.to_lowercase();
        n.ends_with(".txt") || n.ends_with(".log") || n.ends_with(".json") || n.ends_with(".yml") || 
        n.ends_with(".yaml") || n.ends_with(".js") || n.ends_with(".ts") || n.ends_with(".rs") || 
        n.ends_with(".py") || n.ends_with(".php") || n.ends_with(".sh") || n.ends_with(".md")
    };

    let handle_upload = move |files: web_sys::FileList| {
        let vid = vid_stored.get_value();
        let path = current_path.get();
        set_uploading.set(true);
        
        spawn_local(async move {
            for i in 0..files.length() {
                if let Some(file) = files.item(i as u32) {
                    let form = web_sys::FormData::new().unwrap();
                    let _ = form.append_with_str("volume_id", &vid);
                    let _ = form.append_with_str("path", &path);
                    let _ = form.append_with_blob_and_filename("file", &file, &file.name());
                    
                    let window = web_sys::window().unwrap();
                    let init = web_sys::RequestInit::new();
                    init.set_method("POST");
                    init.set_body(&form);
                    
                    // Fire and forget (or handle errors if needed)
                    let _ = window.fetch_with_str_and_init("/api/storage/upload", &init);
                }
            }
            // Small delay to allow Linstor/SSH to process
            gloo_timers::future::TimeoutFuture::new(1000).await;
            set_uploading.set(false);
            files_res.refetch();
        });
    };

    view! {
        <div class="flex flex-col h-full bg-gray-900 rounded-xl border border-gray-800 overflow-hidden">
            // ── Toolbar ──────────────────────────────────────────────────────
            <div class="p-3 border-b border-gray-800 bg-gray-900/50 flex items-center justify-between gap-4">
                <div class="flex items-center gap-2">
                    <button 
                        on:click=move |_| set_show_new_folder.set(true)
                        class="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-md transition-colors"
                        title="New Folder"
                    >
                        <Folder class="w-5 h-5"/>
                    </button>
                    <div class="w-px h-6 bg-gray-800 mx-1"/>
                    <button 
                        class="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-md transition-colors relative"
                        title="Upload"
                    >
                        <Upload class="w-5 h-5"/>
                        <input 
                            type="file" 
                            multiple 
                            class="absolute inset-0 opacity-0 cursor-pointer"
                            on:change=move |ev| {
                                if let Some(target) = ev.target() {
                                    let input = target.unchecked_into::<web_sys::HtmlInputElement>();
                                    if let Some(files) = input.files() {
                                        handle_upload(files);
                                    }
                                }
                            }
                        />
                    </button>
                    {
                        move || if !selected_files.get().is_empty() {
                            view! {
                                <div class="flex items-center gap-2 animate-in fade-in slide-in-from-left-2 transition-all">
                                    <div class="w-px h-6 bg-gray-800 mx-1"/>
                                    <button 
                                        on:click=move |_| {
                                            let vid = vid_stored.get_value();
                                            let paths: Vec<String> = selected_files.get().into_iter()
                                                .map(|f| if current_path.get().is_empty() { f.clone() } else { format!("{}/{}", current_path.get(), f) })
                                                .collect();
                                            delete_action.dispatch(DeleteFiles { volume_id: vid, sub_paths: paths });
                                        }
                                        class="p-2 text-red-400 hover:text-red-300 hover:bg-red-400/10 rounded-md transition-colors"
                                        title="Delete Selected"
                                    >
                                        <Trash class="w-5 h-5"/>
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! { }.into_any()
                        }
                    }
                </div>

                <div class="flex-1 max-w-sm relative">
                    <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" />
                    <input 
                        type="text"
                        placeholder="Search files..."
                        prop:value=search_query
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                        class="w-full bg-gray-950 border border-gray-800 rounded-lg pl-10 pr-4 py-2 text-sm focus:ring-2 focus:ring-blue-500 transition-all outline-none"
                    />
                </div>
            </div>

            // ── Breadcrumbs ──────────────────────────────────────────────────
            <div class="px-4 py-2 bg-gray-950/30 border-b border-gray-800 flex items-center gap-2 text-sm">
                <button on:click=root_nav class="text-gray-500 hover:text-blue-400 transition-colors">"Root"</button>
                {move || {
                    let path = current_path.get();
                    if path.is_empty() { return view! { }.into_any() }
                    let parts: Vec<String> = path.split('/').map(|s| s.to_string()).collect();
                    view! {
                        <div class="flex items-center gap-2">
                            {parts.into_iter().enumerate().map(|(idx, part)| view! {
                                <div class="flex items-center gap-2">
                                    <ChevronRight class="w-3 h-3 text-gray-700"/>
                                    <button 
                                        on:click=move |_| navigate_up(idx)
                                        class="text-gray-400 hover:text-blue-400 transition-colors"
                                    >
                                        {part}
                                    </button>
                                </div>
                            }).collect_view()}
                        </div>
                    }.into_any()
                }}
            </div>

            // ── File List ────────────────────────────────────────────────────
            <div 
                class="flex-1 overflow-y-auto custom-scrollbar relative"
                on:dragover=move |ev: web_sys::DragEvent| {
                    ev.prevent_default();
                    ev.stop_propagation();
                    set_drag_active.set(true);
                }
                on:dragenter=move |ev: web_sys::DragEvent| {
                    ev.prevent_default();
                    ev.stop_propagation();
                    set_drag_active.set(true);
                }
                on:dragleave=move |ev: web_sys::DragEvent| {
                    ev.prevent_default();
                    ev.stop_propagation();
                    set_drag_active.set(false);
                }
                on:drop=move |ev: web_sys::DragEvent| {
                    ev.prevent_default();
                    ev.stop_propagation();
                    set_drag_active.set(false);
                    if let Some(dt) = ev.data_transfer() {
                        if let Some(files) = dt.files() {
                            handle_upload(files);
                        }
                    }
                }
            >
                {move || if drag_active.get() {
                    view! {
                        <div class="absolute inset-0 bg-blue-500/10 border-2 border-dashed border-blue-500/50 m-2 rounded-xl z-50 flex flex-col items-center justify-center backdrop-blur-[2px] animate-in fade-in zoom-in-95 transition-all">
                            <Upload class="w-12 h-12 text-blue-500 mb-4 animate-bounce"/>
                            <p class="text-lg font-bold text-blue-400">"Drop files to upload"</p>
                        </div>
                    }.into_any()
                } else { view! { }.into_any() }}

                <Suspense fallback=|| view! { 
                    <div class="absolute inset-0 flex items-center justify-center bg-gray-900/50 backdrop-blur-sm z-10">
                        <div class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
                    </div>
                }>
                    {move || files_res.get().map(|res: Result<Vec<FileEntry>, ServerFnError>| match res {
                        Ok(files) => {
                            let filtered: Vec<FileEntry> = files.into_iter()
                                .filter(|f| f.name.to_lowercase().contains(&search_query.get().to_lowercase()))
                                .collect();
                            
                            if filtered.is_empty() {
                                return view! {
                                    <div class="flex flex-col items-center justify-center p-20 text-gray-500">
                                        <Folder class="w-16 h-16 mb-4 opacity-20"/>
                                        <p>"This folder is empty"</p>
                                    </div>
                                }.into_any()
                            }

                            view! {
                                <table class="w-full text-left border-collapse">
                                    <thead class="sticky top-0 bg-gray-900 z-10">
                                        <tr class="border-b border-gray-800">
                                            <th class="w-10 p-3">
                                                <input type="checkbox" class="rounded border-gray-700 bg-gray-800"/>
                                            </th>
                                            <th class="p-3 text-xs font-semibold text-gray-400 uppercase tracking-wider">"Name"</th>
                                            <th class="p-3 text-xs font-semibold text-gray-400 uppercase tracking-wider">"Size"</th>
                                            <th class="p-3 text-xs font-semibold text-gray-400 uppercase tracking-wider">"Modified"</th>
                                            <th class="p-3 text-xs font-semibold text-gray-400 uppercase tracking-wider text-right">"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-gray-800/40">
                                        {filtered.into_iter().map(|file| {
                                            let name = file.name.clone();
                                            let is_dir = file.is_dir;
                                            let is_selected = selected_files.get().contains(&name);
                                            
                                            view! {
                                                <tr 
                                                    class=move || format!("group transition-colors hover:bg-gray-800/40 {}", 
                                                        if is_selected { "bg-blue-900/10" } else { "" })
                                                >
                                                    <td class="p-3">
                                                        <input 
                                                            type="checkbox" 
                                                            checked=is_selected
                                                            on:change=move |ev| {
                                                                let checked = event_target_checked(&ev);
                                                                set_selected_files.update(|set| {
                                                                    if checked { set.insert(name.clone()); }
                                                                    else { set.remove(&name); }
                                                                });
                                                            }
                                                            class="rounded border-gray-700 bg-gray-800 text-blue-600 focus:ring-blue-500"
                                                        />
                                                    </td>
                                                    <td class="p-3">
                                                        <div 
                                                            class="flex items-center gap-3 cursor-pointer group"
                                                            on:click={
                                                                let n = file.name.clone();
                                                                move |_| if is_dir { navigate_to(n.clone()) } 
                                                                else if is_text_file(&n) { set_preview_file.set(Some(n.clone())) }
                                                            }
                                                        >
                                                            {if is_dir {
                                                                view! { <Folder class="w-5 h-5 text-blue-400 group-hover:scale-110 transition-transform"/> }.into_any()
                                                            } else {
                                                                view! { <File class="w-5 h-5 text-gray-500 group-hover:scale-110 transition-transform"/> }.into_any()
                                                            }}
                                                            <span class="text-sm font-medium text-gray-200 group-hover:text-blue-400 transition-colors">{file.name.clone()}</span>
                                                        </div>
                                                    </td>
                                                    <td class="p-3 text-xs text-gray-500">
                                                        {if is_dir { "-".to_string() } else { format_size(file.size) }}
                                                    </td>
                                                    <td class="p-3 text-xs text-gray-500">
                                                        {format_date(file.mtime)}
                                                    </td>
                                                    <td class="p-3 text-right">
                                                        <div class="flex items-center justify-end gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                                            <button class="p-1.5 text-gray-500 hover:text-white hover:bg-gray-700 rounded transition-colors" title="Download">
                                                                <Download class="w-4 h-4"/>
                                                            </button>
                                                            <button class="p-1.5 text-gray-500 hover:text-white hover:bg-gray-700 rounded transition-colors" title="Rename">
                                                                <Edit class="w-4 h-4"/>
                                                            </button>
                                                        </div>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <div class="p-10 text-center text-red-400">
                                <p>"Error loading files: " {e.to_string()}</p>
                            </div>
                        }.into_any()
                    })}
                </Suspense>
            </div>

            // ── Upload Overlay ───────────────────────────────────────────────
            {move || if uploading.get() {
                view! {
                    <div class="absolute inset-0 bg-gray-900/40 backdrop-blur-sm flex items-center justify-center z-50">
                        <div class="bg-gray-800 p-6 rounded-2xl border border-gray-700 shadow-2xl flex flex-col items-center gap-4">
                            <div class="animate-spin rounded-full h-10 w-10 border-t-2 border-b-2 border-blue-500"></div>
                            <p class="text-sm font-bold text-white">"Uploading files..."</p>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { }.into_any()
            }}

            // ── New Folder Modal ──
            {
                move || if show_new_folder.get() {
                    let (name, set_name) = signal(String::new());
                    view! {
                        <div class="fixed inset-0 z-[100] flex items-center justify-center p-4">
                            <div class="absolute inset-0 bg-black/70 backdrop-blur-sm" on:click=move |_| set_show_new_folder.set(false)/>
                            <div class="bg-gray-900 border border-gray-800 rounded-2xl p-6 w-full max-w-sm relative z-10 shadow-2xl">
                                <h3 class="text-lg font-bold text-white mb-4">"New Folder"</h3>
                                <input 
                                    type="text" 
                                    prop:value=name
                                    on:input=move |ev| set_name.set(event_target_value(&ev))
                                    placeholder="Folder name"
                                    class="w-full bg-gray-950 border border-gray-800 rounded-lg px-4 py-2 mb-6 outline-none focus:ring-2 focus:ring-blue-500"
                                />
                                <div class="flex justify-end gap-3">
                                    <button on:click=move |_| set_show_new_folder.set(false) class="px-4 py-2 text-sm text-gray-400 hover:text-white">"Cancel"</button>
                                    <button 
                                        on:click=move |_| {
                                            let vid = vid_stored.get_value();
                                            let path = if current_path.get().is_empty() { name.get() } else { format!("{}/{}", current_path.get(), name.get()) };
                                            create_folder_action.dispatch(CreateFolder { volume_id: vid, sub_path: path });
                                            set_show_new_folder.set(false);
                                        }
                                        class="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-bold transition-all"
                                    >
                                        "Create"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else { view! { }.into_any() }
            }

            // ── Preview Modal ──
            {
                move || preview_file.get().map(|name| {
                    let name_clone = name.clone();
                    let vid = vid_stored.get_value();
                    let path = if current_path.get().is_empty() { name.clone() } else { format!("{}/{}", current_path.get(), name.clone()) };
                    
                    let preview = Resource::new(|| (), move |_| {
                        let vid = vid.clone();
                        let path = path.clone();
                        async move { get_file_preview(vid, path).await }
                    });

                    view! {
                        <div class="fixed inset-0 z-[100] flex items-center justify-center p-8">
                            <div class="absolute inset-0 bg-black/80 backdrop-blur-md" on:click=move |_| set_preview_file.set(None)/>
                            <div class="bg-gray-900 border border-gray-800 rounded-2xl w-full max-w-4xl max-h-[80vh] flex flex-col relative z-10 shadow-2xl overflow-hidden">
                                <div class="p-4 border-b border-gray-800 flex items-center justify-between">
                                    <h3 class="text-sm font-mono text-blue-400 font-bold">{name_clone}</h3>
                                    <button on:click=move |_| set_preview_file.set(None) class="p-2 text-gray-500 hover:text-white">
                                        <X class="w-5 h-5"/>
                                    </button>
                                </div>
                                <div class="flex-1 overflow-auto p-4 custom-scrollbar bg-gray-950 font-mono text-sm text-gray-300">
                                    <Suspense fallback=|| view! { <div class="animate-pulse flex flex-col gap-2"> {vec![0; 10].iter().map(|_| view! { <div class="h-4 bg-gray-800 rounded w-full"/> }).collect_view()} </div> }>
                                        {move || preview.get().map(|p: Result<String, ServerFnError>| match p {
                                            Ok(content) => view! { <pre class="whitespace-pre-wrap">{content}</pre> }.into_any(),
                                            Err(e) => view! { <p class="text-red-400">{e.to_string()}</p> }.into_any()
                                        })}
                                    </Suspense>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                })
            }
        </div>
    }
}

fn format_size(bytes: u64) -> String {
    if bytes == 0 { return "0 B".into() }
    let k = 1024.0;
    let sizes = ["B", "KB", "MB", "GB", "TB"];
    let i = (bytes as f64).log(k).floor() as usize;
    format!("{:.1} {}", (bytes as f64) / k.powi(i as i32), sizes[i])
}

fn format_date(_ts: u64) -> String {
    // Basic date formatting
    "Today".to_string()
}
