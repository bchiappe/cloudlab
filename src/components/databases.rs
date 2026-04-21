use leptos::prelude::*;
use crate::databases::*;
use crate::hosts::list_hosts;
use crate::components::icons::{Database, Plus, Trash, Terminal};

#[component]
pub fn DatabasesPage() -> impl IntoView {
    let (show_modal, set_show_modal) = signal(false);
    
    let databases_res = Resource::new(|| (), |_| async { list_databases().await });
    let hosts_res = Resource::new(|| (), |_| async { list_hosts().await });

    let deploy_action = ServerAction::<DeployDatabase>::new();
    let delete_action = ServerAction::<DeleteDatabase>::new();

    let user_ctx = expect_context::<crate::app::UserContext>();
    let is_viewer = move || match user_ctx.get() {
        Some(Ok(Some(u))) => u.role == crate::auth::UserRole::Viewer,
        _ => false,
    };

    view! {
        <div class="space-y-6 animate-in fade-in duration-500">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold tracking-tight text-gray-100">"Managed Databases"</h1>
                    <p class="text-gray-400 mt-1">"Deploy and manage high-availability database clusters via Docker."</p>
                </div>
                <button 
                    on:click=move |_| set_show_modal.set(true)
                    disabled=is_viewer
                    class=move || format!("flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-semibold transition-all shadow-lg shadow-blue-900/20 {}",
                        if is_viewer() { "opacity-50 cursor-not-allowed grayscale" } else { "cursor-pointer" }
                    )
                >
                    <Plus class="w-4 h-4"/>
                    "Deploy Database"
                </button>
            </div>

            <div class="grid grid-cols-1 gap-6">
                <div class="bg-gray-900/50 backdrop-blur-sm border border-gray-800 rounded-xl overflow-hidden">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wider">
                                <th class="px-6 py-4 font-semibold">"Name"</th>
                                <th class="px-6 py-4 font-semibold">"Type"</th>
                                <th class="px-6 py-4 font-semibold">"Host"</th>
                                <th class="px-6 py-4 font-semibold">"Port"</th>
                                <th class="px-6 py-4 font-semibold">"Status"</th>
                                <th class="px-6 py-4 font-semibold text-right">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-800">
                            <Suspense fallback=|| view! { <tr><td colspan="6" class="px-6 py-8 text-center text-gray-500">"Loading databases..."</td></tr> }>
                                {move || databases_res.get().map(|res| match res {
                                    Ok(databases) if databases.is_empty() => view! {
                                        <tr><td colspan="6" class="px-6 py-12 text-center text-gray-500 italic">"No managed databases found. Deploy one to get started."</td></tr>
                                    }.into_any(),
                                    Ok(databases) => databases.into_iter().map(|db| {
                                        let id_del = db.id.clone();
                                        view! {
                                            <tr class="hover:bg-gray-800/30 transition-colors group">
                                                <td class="px-6 py-4">
                                                    <div class="flex items-center gap-3">
                                                        <Database class="w-5 h-5 text-blue-400"/>
                                                        <span class="font-semibold text-gray-100">{db.name}</span>
                                                    </div>
                                                </td>
                                                <td class="px-6 py-4">
                                                    <span class="text-xs font-mono px-2 py-1 bg-gray-800 rounded text-gray-300 uppercase">{db.db_type}</span>
                                                </td>
                                                <td class="px-6 py-4 text-gray-400 text-sm">{db.host_name}</td>
                                                <td class="px-6 py-4 font-mono text-sm text-gray-300">{db.port}</td>
                                                <td class="px-6 py-4">
                                                    <StatusBadge status=db.status />
                                                </td>
                                                <td class="px-6 py-4 text-right">
                                                    <div class="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                                        <button class="p-2 text-gray-400 hover:text-blue-400 hover:bg-blue-400/10 rounded-lg transition-all" title="Console">
                                                            <Terminal class="w-4 h-4"/>
                                                        </button>
                                                         <button 
                                                            on:click=move |_| { delete_action.dispatch(DeleteDatabase { id: id_del.clone() }); }
                                                            disabled=is_viewer
                                                            class=move || format!("p-2 text-gray-400 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-all {}",
                                                                if is_viewer() { "opacity-0 cursor-not-allowed pointer-events-none" } else { "cursor-pointer" }
                                                            )
                                                            title="Delete"
                                                        >
                                                            <Trash class="w-4 h-4"/>
                                                        </button>
                                                    </div>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view().into_any(),
                                    Err(_) => view! { <tr><td colspan="6" class="px-6 py-8 text-center text-red-400">"Error loading databases."</td></tr> }.into_any(),
                                })}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </div>

            {move || show_modal.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-gray-950/80 backdrop-blur-sm animate-in fade-in zoom-in duration-200">
                    <div class="bg-gray-900 border border-gray-800 rounded-2xl shadow-2xl w-full max-w-lg overflow-hidden">
                        <div class="px-8 py-6 border-b border-gray-800 flex items-center justify-between bg-gray-800/30">
                            <h2 class="text-xl font-bold text-gray-100">"Deploy Managed Database"</h2>
                            <button on:click=move |_| set_show_modal.set(false) class="text-gray-400 hover:text-white transition">"✕"</button>
                        </div>
                        
                        <ActionForm action=deploy_action attr:class="p-8 space-y-6" on:submit=move |_| set_show_modal.set(false)>
                            <div class="grid grid-cols-2 gap-6">
                                <div class="space-y-2">
                                    <label class="text-xs font-semibold text-gray-400 uppercase tracking-wider">"Database Name"</label>
                                    <input type="text" name="name" required class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-lg focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all text-sm"/>
                                </div>
                                <div class="space-y-2">
                                    <label class="text-xs font-semibold text-gray-400 uppercase tracking-wider">"Database Type"</label>
                                    <select name="db_type" class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-lg focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all text-sm">
                                        <option value="mysql">"MySQL 8.0"</option>
                                        <option value="mariadb">"MariaDB 11.x"</option>
                                        <option value="postgres">"PostgreSQL 16"</option>
                                        <option value="mssql">"MS SQL Server (Free)"</option>
                                        <option value="oracle">"Oracle Free"</option>
                                        <option value="mongodb">"MongoDB Community"</option>
                                    </select>
                                </div>
                            </div>

                            <div class="grid grid-cols-2 gap-6">
                                <div class="space-y-2">
                                    <label class="text-xs font-semibold text-gray-400 uppercase tracking-wider">"Target Host"</label>
                                    <select name="host_id" class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-lg focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all text-sm">
                                        {move || hosts_res.get().map(|res| match res {
                                            Ok(hosts) => hosts.into_iter().map(|h| view! { <option value=h.id>{h.name}</option> }).collect_view().into_any(),
                                            _ => view! { <option disabled=true>"Loading hosts..."</option> }.into_any()
                                        })}
                                    </select>
                                </div>
                                <div class="space-y-2">
                                    <label class="text-xs font-semibold text-gray-400 uppercase tracking-wider">"External Port"</label>
                                    <input type="number" name="port" value="3306" class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-lg focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all text-sm"/>
                                </div>
                            </div>

                            <div class="space-y-2">
                                <label class="text-xs font-semibold text-gray-400 uppercase tracking-wider">"Root/Admin Password"</label>
                                <input type="password" name="root_password" required class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-lg focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all text-sm" placeholder="••••••••"/>
                            </div>

                            <div class="pt-4 flex items-center justify-end gap-3">
                                <button type="button" on:click=move |_| set_show_modal.set(false) class="px-4 py-2 text-sm font-semibold text-gray-400 hover:text-white transition">"Cancel"</button>
                                <button type="submit" class="px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-semibold transition-all shadow-lg shadow-blue-900/40 cursor-pointer">"Deploy Now"</button>
                            </div>
                        </ActionForm>
                    </div>
                </div>
            })}
        </div>
    }
}

#[component]
fn StatusBadge(status: String) -> impl IntoView {
    let classes = match status.as_str() {
        "online" => "bg-green-500/10 text-green-400 border-green-500/20",
        "offline" => "bg-gray-500/10 text-gray-400 border-gray-500/20",
        "provisioning" => "bg-blue-500/10 text-blue-400 border-blue-500/20 animate-pulse",
        _ => "bg-red-500/10 text-red-400 border-red-500/20",
    };
    view! {
        <span class=format!("text-[10px] sm:text-xs font-bold px-2.5 py-1 rounded-full border uppercase tracking-wider {}", classes)>
            {status}
        </span>
    }
}
