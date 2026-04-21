use leptos::prelude::*;
use crate::components::icons::{
    Desktop, HardDrive, Cube, Database, SignOut, List, MagnifyingGlass, 
    UserCircle, CaretLeft, SquaresFour, Brain, Stack, CloudArrowUp, Gear, Globe
};
use crate::app::UserContext;
use crate::auth::Logout;
use crate::components::job_panel::JobQueuePanel;

#[component]
pub fn DashboardLayout(children: Children) -> impl IntoView {
    let user_ctx = expect_context::<UserContext>();
    let (is_expanded, set_expanded) = signal(false);
    
    let logout_action = ServerAction::<Logout>::new();

    let handle_logout = move |_| {
        logout_action.dispatch(Logout {});
        #[cfg(target_arch = "wasm32")]
        let _ = leptos::prelude::window().location().set_href("/login").unwrap_or(());
    };

    view! {
        <div class="flex h-screen bg-gray-950 text-gray-100 overflow-hidden font-sans">
            
            // Sidebar Navigation
            <aside class=move || format!(
                "flex flex-col flex-shrink-0 transition-all duration-300 ease-in-out border-r border-gray-800 bg-gray-900/50 backdrop-blur-xl z-20 overflow-x-hidden {}",
                if is_expanded.get() { "w-64" } else { "w-20" }
            )>
                <div class="flex items-center justify-between h-16 px-4 border-b border-gray-800">
                    <div class=move || format!("flex items-center gap-3 overflow-hidden whitespace-nowrap transition-opacity duration-300 {}", if is_expanded.get() { "opacity-100" } else { "opacity-0 w-0" })>
                        <SquaresFour class="w-6 h-6 text-blue-500" />
                        <span class="font-bold text-lg tracking-wide text-gray-100">"CloudLab"</span>
                    </div>
                    <button 
                        on:click=move |_| set_expanded.update(|e| *e = !*e)
                        class="p-2 rounded-lg hover:bg-gray-800 text-gray-400 hover:text-gray-200 transition-colors cursor-pointer"
                    >
                        {move || if is_expanded.get() { 
                            view!{ <CaretLeft class="w-5 h-5"/> }.into_any() 
                        } else { 
                            view!{ <List class="w-5 h-5"/> }.into_any() 
                        }}
                    </button>
                </div>
                
                <nav class="flex-1 overflow-y-auto overflow-x-hidden py-4 space-y-2 px-3">
                    <SidebarItem href="/" icon=view!{<SquaresFour/>}.into_any() title="Dashboard" expanded=is_expanded active=true />
                    <SidebarItem href="/hosts" icon=view!{<Desktop/>}.into_any() title="Hosts" expanded=is_expanded active=false />
                    <SidebarItem href="/llms" icon=view!{<Brain/>}.into_any() title="LLMs" expanded=is_expanded active=false />
                    <SidebarItem href="/vms" icon=view!{<Cube/>}.into_any() title="Virtual Machines" expanded=is_expanded active=false />
                    <SidebarItem href="/containers" icon=view!{<Stack/>}.into_any() title="Containers" expanded=is_expanded active=false />
                    <SidebarItem href="/proxy" icon=view!{<Globe/>}.into_any() title="Proxy" expanded=is_expanded active=false />
                    <SidebarItem href="/databases" icon=view!{<Database/>}.into_any() title="Databases" expanded=is_expanded active=false />
                    <SidebarItem href="/backups" icon=view!{<CloudArrowUp/>}.into_any() title="Backups" expanded=is_expanded active=false />
                    <SidebarItem href="/storage" icon=view!{<HardDrive/>}.into_any() title="Storage" expanded=is_expanded active=false />
                    <SidebarItem href="/networking" icon=view!{<Database/>}.into_any() title="Networking" expanded=is_expanded active=false />
                    <div class="pt-4 mt-4 border-t border-gray-800/50">
                        {move || {
                            let is_admin = match user_ctx.get() {
                                Some(Ok(Some(u))) => u.role == crate::auth::UserRole::Admin,
                                _ => false,
                            };
                            is_admin.then(|| view! {
                                <SidebarItem href="/settings" icon=view!{<Gear/>}.into_any() title="Settings" expanded=is_expanded active=false />
                            })
                        }}
                    </div>
                </nav>
            </aside>

            // Main Content Area
            <div class="flex-1 flex flex-col min-w-0 overflow-hidden">
                // Top Header
                <header class="h-16 flex-shrink-0 flex items-center justify-between px-6 border-b border-gray-800 bg-gray-950/80 backdrop-blur-sm z-10">
                    <div class="flex items-center flex-1">
                        // Search bar Mock
                        <div class="relative w-full max-w-md hidden md:block">
                            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                <MagnifyingGlass class="h-4 w-4 text-gray-500"/>
                            </div>
                            <input type="text" placeholder="Search VMs, Hosts, Tasks..." 
                                class="w-full pl-10 pr-4 py-2 border border-gray-800 rounded-lg bg-gray-900/50 text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all"/>
                        </div>
                    </div>
                    
                    <div class="flex items-center gap-6 ml-4">
                        <a href="/profile" class="flex items-center gap-2 cursor-pointer hover:text-white transition group">
                            <UserCircle class="w-6 h-6 text-gray-400 group-hover:text-blue-400 transition" />
                            <span class="text-sm font-medium text-gray-300">
                                {move || match user_ctx.get() {
                                    Some(Ok(Some(u))) => u.username,
                                    _ => String::from("User")
                                }}
                            </span>
                        </a>
                        <div class="h-6 w-px bg-gray-800"></div>
                        <button on:click=handle_logout class="flex items-center gap-2 text-sm font-medium text-gray-400 hover:text-red-400 transition cursor-pointer">
                            <SignOut class="w-5 h-5"/>
                            <span class="hidden sm:inline">"Sign Out"</span>
                        </button>
                    </div>
                </header>

                // Dynamic Page Content
                <main class="flex-1 overflow-auto p-6 md:p-8 bg-gray-950 pb-20">
                    {children()}
                </main>
                
                // Live Job Monitoring Panel
                <JobQueuePanel />
            </div>
        </div>
    }
}

#[component]
fn SidebarItem(href: &'static str, icon: AnyView, title: &'static str, expanded: ReadSignal<bool>, active: bool) -> impl IntoView 
{
    let base_classes = "flex items-center gap-4 px-3 py-3 rounded-lg font-medium transition-all duration-200 group relative";
    let active_classes = if active {
        "text-white bg-blue-600/10 border border-blue-500/20 shadow-[inset_4px_0_0_0_rgba(59,130,246,1)]"
    } else {
        "text-gray-400 hover:text-gray-100 hover:bg-gray-800/80 border border-transparent"
    };

    view! {
        <a href=href title=title class=format!("{} {}", base_classes, active_classes)>
            <div class=move || format!("flex-shrink-0 transition-all duration-300 {} {}", 
                if active { "text-blue-500" } else { "text-gray-400 group-hover:text-gray-200" },
                if expanded.get() { "" } else { "mx-auto" }
            )>
                <div class="w-5 h-5">{icon}</div>
            </div>
            
            <span class=move || format!("whitespace-nowrap transition-all duration-300 {}", 
                if expanded.get() { "opacity-100 w-auto ml-4" } else { "opacity-0 w-0 h-0 overflow-hidden pointer-events-none" }
            )>
                {title}
            </span>
        </a>
    }
}
