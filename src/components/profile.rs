use leptos::prelude::*;
use crate::app::UserContext;
use crate::auth::ChangePassword;

#[component]
pub fn ProfilePage() -> impl IntoView {
    let user_ctx = expect_context::<UserContext>();
    let change_password_action = ServerAction::<ChangePassword>::new();
    
    let username = move || match user_ctx.get() {
        Some(Ok(Some(u))) => u.username,
        _ => "User".to_string(),
    };

    view! {
        <div class="max-w-4xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div>
                <h1 class="text-3xl font-bold tracking-tight text-gray-100">"User Profile"</h1>
                <p class="text-gray-400 mt-2">"Manage your account settings and security preferences."</p>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
                // Account Sidebar
                <div class="space-y-6">
                    <div class="bg-gray-900 border border-gray-800 rounded-2xl p-6 text-center space-y-4 shadow-xl">
                        <div class="w-24 h-24 bg-blue-600 rounded-full mx-auto flex items-center justify-center text-3xl font-bold text-white shadow-lg shadow-blue-900/40">
                            {move || username().chars().next().unwrap_or('U').to_uppercase().to_string()}
                        </div>
                        <div>
                            <h2 class="text-xl font-bold text-gray-100">{username}</h2>
                            <p class="text-sm text-gray-500 uppercase tracking-widest font-semibold mt-1">
                                {move || match user_ctx.get() {
                                    Some(Ok(Some(u))) => format!("{:?}", u.role),
                                    _ => "Unknown Role".to_string(),
                                }}
                            </p>
                        </div>
                        <div class="pt-4 flex flex-col gap-2">
                             <button class="w-full py-2 px-4 bg-gray-800 hover:bg-gray-700 text-gray-200 rounded-lg text-sm font-medium transition cursor-pointer">"Upload Avatar"</button>
                             <button class="w-full py-2 px-4 text-red-400 hover:text-red-300 text-sm font-medium transition cursor-pointer">"Deactivate Account"</button>
                        </div>
                    </div>

                    <div class="bg-gray-900 border border-gray-800 rounded-2xl p-6 shadow-xl space-y-4">
                        <h3 class="font-bold text-gray-100 uppercase tracking-wider text-xs">"System Permissions"</h3>
                        <div class="space-y-3">
                            <div class="flex items-center justify-between">
                                <span class="text-sm text-gray-400">"Read-only Access"</span>
                                <span class="w-2 h-2 rounded-full bg-green-500 shadow-lg shadow-green-500/50"></span>
                            </div>
                            <div class="flex items-center justify-between">
                                <span class="text-sm text-gray-400">"Infra Management"</span>
                                {move || {
                                    let is_op = match user_ctx.get() {
                                        Some(Ok(Some(u))) => u.role.level() >= crate::auth::UserRole::Operator.level(),
                                        _ => false,
                                    };
                                    view! { <span class=format!("w-2 h-2 rounded-full {}", if is_op { "bg-green-500 shadow-lg shadow-green-500/50" } else { "bg-gray-600" })></span> }
                                }}
                            </div>
                            <div class="flex items-center justify-between">
                                <span class="text-sm text-gray-400">"User Management"</span>
                                {move || {
                                    let is_admin = match user_ctx.get() {
                                        Some(Ok(Some(u))) => u.role == crate::auth::UserRole::Admin,
                                        _ => false,
                                    };
                                    view! { <span class=format!("w-2 h-2 rounded-full {}", if is_admin { "bg-green-500 shadow-lg shadow-green-500/50" } else { "bg-gray-600" })></span> }
                                }}
                            </div>
                        </div>
                    </div>
                </div>

                // Main Settings Area
                <div class="lg:col-span-2 space-y-8">
                    // General Info
                    <div class="bg-gray-900 border border-gray-800 rounded-2xl shadow-xl overflow-hidden">
                        <div class="px-8 py-6 border-b border-gray-800 bg-gray-800/30">
                            <h2 class="text-xl font-bold text-gray-100">"Security Settings"</h2>
                        </div>
                        
                        <div class="p-8 space-y-8">
                            // Change Password
                            <div class="space-y-6">
                                <h3 class="text-lg font-semibold text-gray-100 flex items-center gap-2">
                                    <span class="w-1.5 h-6 bg-blue-500 rounded-full"></span>
                                    "Change Password"
                                </h3>
                                
                                <ActionForm action=change_password_action attr:class="space-y-6">
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                        <div class="space-y-2">
                                            <label class="text-xs font-semibold text-gray-400 uppercase tracking-wider">"Current Password"</label>
                                            <input type="password" name="current_password" required class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-lg focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all text-sm"/>
                                        </div>
                                    </div>
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                        <div class="space-y-2">
                                            <label class="text-xs font-semibold text-gray-400 uppercase tracking-wider">"New Password"</label>
                                            <input type="password" name="new_password" required class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-lg focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all text-sm"/>
                                        </div>
                                        <div class="space-y-2">
                                            <label class="text-xs font-semibold text-gray-400 uppercase tracking-wider">"Confirm New Password"</label>
                                            <input type="password" name="confirm_password" required class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-lg focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all text-sm"/>
                                        </div>
                                    </div>
                                    
                                    <div class="pt-2">
                                        <button type="submit" class="px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-semibold transition-all shadow-lg shadow-blue-900/40 cursor-pointer">
                                            "Update Password"
                                        </button>
                                        
                                        <Suspense fallback=|| ()>
                                            {move || change_password_action.value().get().map(|res| match res {
                                                Ok(_) => view! { <p class="text-green-400 text-sm mt-4 font-medium animate-in fade-in ease-out">"✓ Password updated successfully."</p> }.into_any(),
                                                Err(e) => view! { <p class="text-red-400 text-sm mt-4 font-medium">"✗ Error: " {e.to_string()}</p> }.into_any(),
                                            })}
                                        </Suspense>
                                    </div>
                                </ActionForm>
                            </div>

                            <div class="h-px bg-gray-800 w-full"></div>

                            // Two-Factor Auth
                            <div class="flex items-center justify-between p-4 bg-gray-950/50 rounded-xl border border-gray-800 group hover:border-blue-500/20 transition-colors">
                                <div class="space-y-1">
                                    <h3 class="font-bold text-gray-100">"Two-Factor Authentication"</h3>
                                    <p class="text-sm text-gray-500">"Add an extra layer of security to your account."</p>
                                </div>
                                <button class="px-4 py-2 bg-gray-800 hover:bg-gray-700 text-gray-200 rounded-lg text-xs font-bold transition uppercase tracking-widest cursor-pointer">"Enable"</button>
                            </div>

                            // Session Mgmt
                            <div class="flex items-center justify-between p-4 bg-gray-950/50 rounded-xl border border-gray-800 group hover:border-red-500/20 transition-colors">
                                <div class="space-y-1">
                                    <h3 class="font-bold text-gray-100">"Active Sessions"</h3>
                                    <p class="text-sm text-gray-500">"Manage devices currently logged in to your account."</p>
                                </div>
                                <button class="px-4 py-2 border border-red-500/20 text-red-400 hover:bg-red-500/10 rounded-lg text-xs font-bold transition uppercase tracking-widest cursor-pointer">"Revoke All"</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
