use leptos::prelude::*;
use crate::settings::*;
use crate::auth::*;
use crate::components::icons::{Gear, Plus, Trash, UserCircle, CheckCircle, WarningCircle};

pub mod api_keys;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("ha"); // "ha", "users", or "keys"
    
    view! {
        <div class="space-y-6 animate-in fade-in duration-500">
            <div>
                <h1 class="text-3xl font-bold tracking-tight text-gray-100 italic">"Cluster Settings"</h1>
                <p class="text-gray-400 mt-1">"Manage High Availability configurations and administrative access."</p>
            </div>

            <div class="flex items-center gap-1 p-1 bg-gray-900/50 border border-gray-800 rounded-xl w-fit">
                <button 
                    on:click=move |_| set_active_tab.set("ha")
                    class=move || format!("px-4 py-2 text-sm font-semibold rounded-lg transition-all {}", 
                        if active_tab.get() == "ha" { "bg-blue-600 text-white shadow-lg" } else { "text-gray-400 hover:text-gray-200" })
                >
                    "Cluster HA"
                </button>
                <button 
                    on:click=move |_| set_active_tab.set("users")
                    class=move || format!("px-4 py-2 text-sm font-semibold rounded-lg transition-all {}", 
                        if active_tab.get() == "users" { "bg-blue-600 text-white shadow-lg" } else { "text-gray-400 hover:text-gray-200" })
                >
                    "User Management"
                </button>
                <button 
                    on:click=move |_| set_active_tab.set("keys")
                    class=move || format!("px-4 py-2 text-sm font-semibold rounded-lg transition-all {}", 
                        if active_tab.get() == "keys" { "bg-blue-600 text-white shadow-lg" } else { "text-gray-400 hover:text-gray-200" })
                >
                    "AI API Keys"
                </button>
            </div>

            <div class="mt-8">
                {move || match active_tab.get() {
                    "ha" => view! { <SyncSettingsTab /> }.into_any(),
                    "users" => view! { <UserManagementTab /> }.into_any(),
                    "keys" => view! { <api_keys::ApiKeysManager /> }.into_any(),
                    _ => view! { <div /> }.into_any(),
                }.into_any()}
            </div>
        </div>
    }
}

#[component]
fn SyncSettingsTab() -> impl IntoView {
    let settings_res = Resource::new(|| (), |_| async { get_settings().await });
    let update_action = ServerAction::<UpdateSettings>::new();

    view! {
        <Suspense fallback=|| view! { <div class="p-12 text-center text-gray-500 italic">"Loading cluster configuration..."</div> }>
            {move || settings_res.get().map(|res| match res {
                Ok(s) => view! {
                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                        <section class="bg-gray-900/40 border border-gray-800 rounded-2xl p-8 space-y-6">
                            <div class="flex items-center gap-3 mb-2">
                                <div class="p-2 bg-blue-500/10 rounded-lg">
                                    <Gear class="w-5 h-5 text-blue-400"/>
                                </div>
                                <h3 class="text-lg font-bold text-gray-100">"HA Configuration"</h3>
                            </div>
                            
                            <ActionForm action=update_action attr:class="space-y-6">
                                <div class="flex items-center justify-between p-4 bg-gray-950/50 border border-gray-800 rounded-xl">
                                    <div>
                                        <p class="font-semibold text-gray-200">"API Replication HA"</p>
                                        <p class="text-xs text-gray-500 mt-0.5">"Enable real-time data syncing between management nodes."</p>
                                    </div>
                                    <label class="relative inline-flex items-center cursor-pointer">
                                        <input type="checkbox" name="settings[ha_enabled]" value="true" checked=s.ha_enabled class="sr-only peer"/>
                                        <div class="w-11 h-6 bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                                    </label>
                                </div>

                                <div class="grid grid-cols-1 gap-6">
                                    <div class="space-y-2">
                                        <label class="text-xs font-bold text-gray-500 uppercase tracking-widest">"Cluster Display Name"</label>
                                        <input type="text" name="settings[cluster_name]" value=s.cluster_name class="w-full px-4 py-2.5 bg-gray-950 border border-gray-800 rounded-xl focus:ring-1 focus:ring-blue-500 transition-all text-sm"/>
                                    </div>
                                    
                                    <div class="grid grid-cols-2 gap-4">
                                        <div class="space-y-2">
                                            <label class="text-xs font-bold text-gray-500 uppercase tracking-widest">"Sync Interval (sec)"</label>
                                            <input type="number" name="settings[sync_interval]" value=s.sync_interval.to_string() class="w-full px-4 py-2.5 bg-gray-950 border border-gray-800 rounded-xl focus:ring-1 focus:ring-blue-500 transition-all text-sm"/>
                                        </div>
                                        <div class="space-y-2">
                                            <label class="text-xs font-bold text-gray-500 uppercase tracking-widest">"Secondary Node IP"</label>
                                            <input type="text" name="settings[secondary_node_ip]" value=s.secondary_node_ip placeholder="10.0.0.2" class="w-full px-4 py-2.5 bg-gray-950 border border-gray-800 rounded-xl focus:ring-1 focus:ring-blue-500 transition-all text-sm"/>
                                        </div>
                                    </div>
                                </div>

                                <div class="pt-4 flex items-center justify-between">
                                    <div class="flex items-center gap-2 text-xs text-gray-500 italic">
                                        {move || (update_action.version().get() > 0).then(|| view! { <CheckCircle class="w-3 h-3 text-green-500" /> })}
                                        {move || if update_action.pending().get() { "Saving..." } else if update_action.version().get() > 0 { "Configuration Saved" } else { "" }}
                                    </div>
                                    <button type="submit" class="px-6 py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-xl font-bold transition-all shadow-lg shadow-blue-900/20 cursor-pointer">
                                        "Save Config"
                                    </button>
                                </div>
                            </ActionForm>
                        </section>

                        <div class="space-y-6">
                             <div class="p-6 bg-blue-900/10 border border-blue-800/20 rounded-2xl flex gap-4">
                                <div class="p-2 bg-blue-500/20 rounded-lg h-fit">
                                    <Gear class="w-5 h-5 text-blue-400"/>
                                </div>
                                <div>
                                    <h4 class="font-bold text-gray-100 underline decoration-blue-500/50 underline-offset-4">"Sync Strategy"</h4>
                                    <p class="text-sm text-gray-400 mt-2 leading-relaxed">
                                        "Data is replicated across management nodes via Cloudlab's proprietary API sync mechanism. Ensure that port " <code class="text-blue-400 bg-blue-500/10 px-1 rounded">"3000"</code> " is open for inter-node communication."
                                    </p>
                                </div>
                             </div>

                             <div class="p-6 bg-purple-900/10 border border-purple-800/20 rounded-2xl flex gap-4">
                                <div class="p-2 bg-purple-500/20 rounded-lg h-fit">
                                    <WarningCircle class="w-5 h-5 text-purple-400"/>
                                </div>
                                <div>
                                    <h4 class="font-bold text-gray-100 underline decoration-purple-500/50 underline-offset-4">"HA Quorum"</h4>
                                    <p class="text-sm text-gray-400 mt-2 leading-relaxed">
                                        "A minimum of 2 management nodes is required for active-active redundancy. Secondary node IP must be reachable from the primary node."
                                    </p>
                                </div>
                             </div>
                        </div>
                    </div>
                }.into_any(),
                Err(_) => view! { <div class="p-12 text-red-400">"Error loading settings."</div> }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn UserManagementTab() -> impl IntoView {
    let users_res = Resource::new(|| (), |_| async { list_users().await });
    let create_action = ServerAction::<CreateUser>::new();
    let delete_action = ServerAction::<DeleteUser>::new();
    let update_role_action = ServerAction::<UpdateUserRole>::new();
    let (show_modal, set_show_modal) = signal(false);

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h3 class="text-xl font-bold text-gray-100">"Administrative Users"</h3>
                    <p class="text-sm text-gray-500 mt-1">"Manage access for system administrators."</p>
                </div>
                <button 
                    on:click=move |_| set_show_modal.set(true)
                    class="flex items-center gap-2 px-4 py-2 bg-gray-800 hover:bg-gray-700 text-gray-200 rounded-lg font-semibold transition-all border border-gray-700 cursor-pointer"
                >
                    <Plus class="w-4 h-4"/>
                    "Create User"
                </button>
            </div>

            <div class="bg-gray-900/40 border border-gray-800 rounded-2xl overflow-hidden shadow-xl">
                <table class="w-full text-left">
                    <thead>
                        <tr class="bg-gray-800/50 border-b border-gray-800 text-xs font-bold text-gray-500 uppercase tracking-widest">
                            <th class="px-8 py-4">"Username"</th>
                            <th class="px-8 py-4">"User ID"</th>
                            <th class="px-8 py-4">"Role / Permissions"</th>
                            <th class="px-8 py-4 text-right">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-gray-800/60">
                        <Suspense fallback=|| view! { <tr><td colspan="4" class="px-8 py-12 text-center text-gray-600 italic">"Fetching administrators..."</td></tr> }>
                            {move || users_res.get().map(|res| match res {
                                Ok(users) => users.into_iter().map(|u| {
                                    let id_del = u.id.clone();
                                    let is_admin = u.username == "admin";
                                    view! {
                                        <tr class="hover:bg-gray-800/20 transition-colors group">
                                            <td class="px-8 py-5">
                                                <div class="flex items-center gap-3">
                                                    <div class="p-2 bg-gray-800 rounded-lg group-hover:bg-blue-500/10 transition-colors">
                                                        <UserCircle class="w-5 h-5 text-gray-500 group-hover:text-blue-400 transition-colors"/>
                                                    </div>
                                                    <span class="font-bold text-gray-100">{u.username.clone()}</span>
                                                </div>
                                            </td>
                                            <td class="px-8 py-5 text-xs font-mono text-gray-500">{u.id.chars().take(8).collect::<String>()}</td>
                                            <td class="px-8 py-5">
                                                {let u_id = u.id.clone();
                                                 let u_role = u.role.clone();
                                                 let is_root = u.username == "admin";
                                                 view! {
                                                    <div class="flex items-center gap-2">
                                                        <select 
                                                            disabled=is_root
                                                            on:change=move |ev| {
                                                                let val = event_target_value(&ev);
                                                                update_role_action.dispatch(UpdateUserRole { 
                                                                    id: u_id.clone(), 
                                                                    role: UserRole::from(val) 
                                                                });
                                                            }
                                                            class=format!("text-[10px] font-bold px-2 py-1 rounded-lg border uppercase tracking-widest transition-colors bg-gray-950 border-gray-800 focus:ring-1 focus:ring-blue-500 outline-none {}", 
                                                                match u_role {
                                                                    UserRole::Admin => "text-purple-400 border-purple-500/20",
                                                                    UserRole::Operator => "text-blue-400 border-blue-500/20",
                                                                    UserRole::Viewer => "text-gray-400 border-gray-500/20",
                                                                }
                                                            )
                                                        >
                                                            <option value="Admin" selected=u_role == UserRole::Admin>"Admin"</option>
                                                            <option value="Operator" selected=u_role == UserRole::Operator>"Operator"</option>
                                                            <option value="Viewer" selected=u_role == UserRole::Viewer>"Viewer"</option>
                                                        </select>
                                                        {is_root.then(|| view! { <span class="text-[8px] text-gray-600 font-mono">"(Locked)"</span> })}
                                                    </div>
                                                }}
                                            </td>
                                            <td class="px-8 py-5 text-right">
                                                {move || (!is_admin).then(|| {
                                                    let id_del_it = id_del.clone();
                                                    view! {
                                                        <button 
                                                            on:click=move |_| { delete_action.dispatch(DeleteUser { id: id_del_it.clone() }); }
                                                            class="p-2 text-gray-500 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-all opacity-0 group-hover:opacity-100"
                                                        >
                                                            <Trash class="w-4 h-4"/>
                                                        </button>
                                                    }
                                                })}
                                            </td>
                                        </tr>
                                    }
                                }).collect_view().into_any(),
                                Err(_) => view! { <tr><td colspan="4" class="px-8 py-12 text-red-400 text-center">"Failed to load user list"</td></tr> }.into_any(),
                            })}
                        </Suspense>
                    </tbody>
                </table>
            </div>

            {move || show_modal.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-gray-950/80 backdrop-blur-sm animate-in fade-in zoom-in duration-200">
                    <div class="bg-gray-900 border border-gray-800 rounded-2xl shadow-3xl w-full max-w-sm overflow-hidden">
                        <div class="px-8 py-6 border-b border-gray-800 flex items-center justify-between bg-gray-800/30">
                            <h2 class="text-xl font-bold text-gray-100 italic">"Create User"</h2>
                            <button on:click=move |_| set_show_modal.set(false) class="text-gray-400 hover:text-white transition cursor-pointer">"✕"</button>
                        </div>
                        
                        <ActionForm action=create_action attr:class="p-8 space-y-6" on:submit=move |_| set_show_modal.set(false)>
                            <div class="space-y-2">
                                <label class="text-xs font-bold text-gray-500 uppercase tracking-widest">"Username"</label>
                                <input type="text" name="username" required class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-xl focus:ring-1 focus:ring-blue-500 transition-all text-sm" placeholder="john.doe"/>
                            </div>
                            <div class="space-y-2">
                                <label class="text-xs font-bold text-gray-500 uppercase tracking-widest">"Temp Password"</label>
                                <input type="password" name="password" required class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-xl focus:ring-1 focus:ring-blue-500 transition-all text-sm"/>
                            </div>
                            
                            <div class="space-y-2">
                                <label class="text-xs font-bold text-gray-500 uppercase tracking-widest">"Initial Role"</label>
                                <select name="role" class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-xl focus:ring-1 focus:ring-blue-500 transition-all text-sm">
                                    <option value="Viewer">"Viewer (Read Only)"</option>
                                    <option value="Operator">"Operator (Manage Infra)"</option>
                                    <option value="Admin">"Admin (Full Access)"</option>
                                </select>
                            </div>
                            
                            <div class="pt-4 flex items-center justify-end gap-3">
                                <button type="button" on:click=move |_| set_show_modal.set(false) class="px-4 py-2 text-sm font-semibold text-gray-400 hover:text-white transition">"Cancel"</button>
                                <button type="submit" class="px-6 py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-xl font-bold transition-all shadow-lg shadow-blue-900/40 cursor-pointer">"Create User"</button>
                            </div>
                        </ActionForm>
                    </div>
                </div>
            })}
        </div>
    }
}
