use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, ParentRoute, Router, Routes, Outlet},
    path,
};
use crate::auth::*;
use crate::components::layout::DashboardLayout;
use crate::components::dashboard::DashboardOverview;
use crate::components::hosts::HostsPage;
use crate::components::vms::VMsPage;
use crate::components::llms::LLMsPage;
use crate::components::containers::ContainersPage;
use crate::components::databases::DatabasesPage;
use crate::components::backups::BackupsPage;
use crate::components::jobs::JobsPage;
use crate::components::profile::ProfilePage;
use crate::components::settings::SettingsPage;
use crate::components::proxy::ProxyPage;
use crate::components::storage::StoragePage;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
                <Stylesheet id="leptos" href="/pkg/cloudlab.css"/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

pub type UserContext = Resource<Result<Option<User>, ServerFnError>>;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let user = Resource::new(|| (), |_| async { get_user().await });
    provide_context::<UserContext>(user);

    view! {
        <Title text="CloudLab Dashboard"/>

        <Router>
            <div class="h-screen bg-gray-950 text-gray-100 flex flex-col font-sans overflow-hidden">
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/login") view=LoginPage/>
                    
                    <ParentRoute path=path!("/") view=SecureLayout>
                        <Route path=path!("") view=DashboardOverview/>
                        <Route path=path!("hosts") view=HostsPage/>
                        <Route path=path!("llms") view=LLMsPage/>
                        <Route path=path!("vms") view=VMsPage/>
                        <Route path=path!("containers") view=ContainersPage/>
                        <Route path=path!("databases") view=DatabasesPage/>
                        <Route path=path!("backups") view=BackupsPage/>
                        <Route path=path!("jobs") view=JobsPage/>
                        <Route path=path!("profile") view=ProfilePage/>
                        <Route path=path!("settings") view=SettingsPage/>
                        <Route path=path!("proxy") view=ProxyPage/>
                        <Route path=path!("storage") view=StoragePage/>
                        <Route path=path!("networking") view=move || view! { <Placeholder title="Virtual Networks" /> }/>
                    </ParentRoute>
                </Routes>
            </div>
        </Router>
    }
}

#[component]
fn SecureLayout() -> impl IntoView {
    let user = expect_context::<UserContext>();

    view! {
        <Suspense fallback=|| view! { <div class="w-full h-screen flex justify-center items-center">"Checking session..."</div> }>
            {
                move || match user.get() {
                    None => view! { }.into_any(),
                    Some(Ok(Some(_))) => view! {
                        <DashboardLayout>
                            <Outlet/>
                        </DashboardLayout>
                    }.into_any(),
                    Some(_) => {
                        view! { <leptos_router::components::Redirect path="/login"/> }.into_any()
                    },
                }
            }
        </Suspense>
    }
}

#[component]
fn Placeholder(title: &'static str) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center h-full text-gray-400">
            <crate::components::icons::Wrench class="w-16 h-16 mb-4 text-gray-500 opacity-50"/>
            <h1 class="text-2xl font-bold text-gray-300">{title}</h1>
            <p class="mt-2 text-sm">"This management interface is currently under construction."</p>
        </div>
    }
}

#[component]
fn LoginPage() -> impl IntoView {
    let login_action = ServerAction::<Login>::new();
    let user = expect_context::<UserContext>();

    Effect::new(move |_| {
        if let Some(Ok(Some(_))) = user.get() {
            #[cfg(target_arch = "wasm32")]
            let _ = leptos::prelude::window().location().set_href("/").unwrap_or(());
        }
    });

    view! {
        <div class="min-h-screen w-full flex items-center justify-center relative overflow-hidden bg-gray-950 w-full">
            <div class="absolute inset-0 z-0">
                <div class="absolute top-1/4 left-1/4 w-96 h-96 bg-purple-600 rounded-full mix-blend-multiply filter blur-3xl opacity-20 animate-pulse"></div>
                <div class="absolute top-1/3 right-1/4 w-96 h-96 bg-blue-600 rounded-full mix-blend-multiply filter blur-3xl opacity-20 animate-pulse"></div>
            </div>

            <div class="w-full max-w-md p-8 bg-gray-900/80 backdrop-blur-xl rounded-2xl shadow-2xl border border-gray-800 z-10 mx-4">
                <div class="text-center mb-8">
                    <h1 class="text-3xl font-extrabold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-purple-500 tracking-tight">"Welcome Back"</h1>
                    <p class="text-sm text-gray-400 mt-2">"Sign in to your Cloudlab Workspace"</p>
                </div>

                <ActionForm action=login_action attr:class="space-y-6">
                    <div>
                        <label for="username" class="block text-sm font-medium text-gray-300 mb-1">"Username"</label>
                        <input id="username" name="username" type="text" required 
                            class="w-full px-4 py-3 bg-gray-950/50 border border-gray-700/80 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-gray-100 placeholder-gray-500 transition duration-200"
                            placeholder="admin"/>
                    </div>
                    <div>
                        <label for="password" class="block text-sm font-medium text-gray-300 mb-1">"Password"</label>
                        <input id="password" name="password" type="password" required 
                            class="w-full px-4 py-3 bg-gray-950/50 border border-gray-700/80 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-gray-100 placeholder-gray-500 transition duration-200"
                            placeholder="••••••••"/>
                    </div>
                    
                    <button type="submit" 
                        class="w-full py-3 px-4 flex justify-center items-center text-sm font-semibold rounded-lg text-white bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-500 hover:to-purple-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 focus:ring-offset-gray-900 transition-all duration-300 transform active:scale-[0.98] shadow-lg cursor-pointer">
                        "Sign In"
                    </button>
                    
                    <div class="text-center text-sm text-red-400 font-medium h-4 mt-2">
                       <ErrorBoundary fallback=|e| view! { <p class="text-red-400">{move || e.get().into_iter().next().map(|(_, err)| err.to_string())}</p> }>
                           {move || login_action.value().get().map(|res| match res {
                               Ok(_) => { 
                                   #[cfg(target_arch = "wasm32")]
                                   let _ = leptos::prelude::window().location().set_href("/").unwrap_or(());
                                   "Connecting...".to_string() 
                               },
                               Err(_) => "".to_string(),
                           })}
                       </ErrorBoundary>
                    </div>
                </ActionForm>
            </div>
        </div>
    }
}
