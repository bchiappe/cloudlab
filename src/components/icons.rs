use leptos::prelude::*;

#[component]
pub fn SquaresFour(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M104,48H56A16,16,0,0,0,40,64v48a16,16,0,0,0,16,16h48a16,16,0,0,0,16-16V64A16,16,0,0,0,104,48Zm0,64H56V64h48Zm96-64H152a16,16,0,0,0-16,16v48a16,16,0,0,0,16,16h48a16,16,0,0,0,16-16V64A16,16,0,0,0,200,48Zm0,64H152V64h48ZM104,144H56a16,16,0,0,0-16,16v48a16,16,0,0,0,16,16h48a16,16,0,0,0,16-16V160A16,16,0,0,0,104,144Zm0,64H56V160h48Zm96-64H152a16,16,0,0,0-16,16v48a16,16,0,0,0,16,16h48a16,16,0,0,0,16-16V160A16,16,0,0,0,200,144Zm0,64H152V160h48Z"/></svg> }
}

#[component]
pub fn Desktop(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M208,40H48A24,24,0,0,0,24,64V176a24,24,0,0,0,24,24H112v24H80a8,8,0,0,0,0,16h96a8,8,0,0,0,0-16H144V200h64a24,24,0,0,0,24-24V64A24,24,0,0,0,208,40Zm8,136a8,8,0,0,1-8,8H48a8,8,0,0,1-8-8V64a8,8,0,0,1,8-8H208a8,8,0,0,1,8,8Z"/></svg> }
}

#[component]
pub fn Cube(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M128,24.12l-96,52.47V174.5l96,57.38,96-57.38V76.59ZM48,89.5,116,52.33v66.4l-68,36.93Zm16,91L120,214.28V134.78l-56-30.82Zm80,33.78L88,172V92.2L144,123ZM208,180.5l-56-30.82v-65l68-36.93v95.32Z"/></svg> }
}

#[component]
pub fn HardDrive(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M208,32H48A16,16,0,0,0,32,48V208a16,16,0,0,0,16,16H208a16,16,0,0,0,16-16V48A16,16,0,0,0,208,32Zm0,176H48V160H208v48Zm0-64H48V48H208Zm-44,28a12,12,0,1,1-12-12A12,12,0,0,1,164,172Z"/></svg> }
}

#[component]
pub fn Database(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M224,192c0,35.35-43,64-96,64s-96-28.65-96-64v-16c0-35.35,43-64,96-64s96,28.65,96,64Zm-16,0c0-10-18.7-21.72-46.61-33.15C158,160.25,155,161.5,152,163c-38.33,18.06-96,18.06-96,0-3-1.5-6-2.75-9.39-4.15C18.7,170.28,0,182,0,192c0,26.51,43,48,96,48S192,218.51,192,192Zm16-80c0-35.35-43-64-96-64S16,76.65,16,112c0,16.59,9.45,31.75,26.11,43.27C44,152,47.4,149.2,51,146s6-6.19,8.46-8.79l6.39,6.59C75,154,85.25,162,96,162c10.51,0,20.53-7.79,29.74-17.76l6.81-6.88c2.47,2.5,4.8,4.91,6.85,7.18s4.65,5.1,7,8c16.31-10.42,26.12-25.07,26.12-40.48Zm-96-48c44.18,0,80,21.49,80,48s-35.82,48-80,48-80-21.49-80-48S83.82,64,128,64Z"/></svg> }
}

#[component]
pub fn List(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M224,128a8,8,0,0,1-8,8H40a8,8,0,0,1,0-16H216A8,8,0,0,1,224,128ZM40,72H216a8,8,0,0,0,0-16H40a8,8,0,0,0,0,16ZM216,184H40a8,8,0,0,0,0,16H216a8,8,0,0,0,0-16Z"/></svg> }
}

#[component]
pub fn CaretLeft(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M165.66,202.34a8,8,0,0,1-11.32,11.32l-80-80a8,8,0,0,1,0-11.32l80-80a8,8,0,0,1,11.32,11.32L91.31,128Z"/></svg> }
}

#[component]
pub fn MagnifyingGlass(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M229.66,218.34l-50.07-50.06a88.11,88.11,0,1,0-11.31,11.31l50.06,50.07a8,8,0,0,0,11.32-11.32ZM40,112a72,72,0,1,1,72,72A72.08,72.08,0,0,1,40,112Z"/></svg> }
}

#[component]
pub fn UserCircle(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24ZM74.08,197.5a64,64,0,0,1,107.84,0,87.83,87.83,0,0,1-107.84,0ZM96,120a32,32,0,1,1,32,32A32,32,0,0,1,96,120Zm97.76,66.41a79.66,79.66,0,0,0-36.06-28.75,48,48,0,1,0-59.4,0,79.66,79.66,0,0,0-36.06,28.75,88,88,0,1,1,131.52,0Z"/></svg> }
}

#[component]
pub fn SignOut(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M120,216a8,8,0,0,1-8,8H48a8,8,0,0,1-8-8V40a8,8,0,0,1,8-8h64a8,8,0,0,1,0,16H56V208h56A8,8,0,0,1,120,216Zm109.66-93.66-40-40a8,8,0,0,0-11.32,11.32L204.69,120H104a8,8,0,0,0,0,16H204.69l-26.35,26.34a8,8,0,0,0,11.32,11.32l40-40A8,8,0,0,0,229.66,122.34Z"/></svg> }
}

#[component]
pub fn CheckCircle(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24Zm45.66,85.66-56,56a8,8,0,0,1-11.32,0l-24-24a8,8,0,0,1,11.32-11.32L112,148.69l50.34-50.35a8,8,0,0,1,11.32,11.32Z"/></svg> }
}

#[component]
pub fn Cpu(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M216,88H200V72a16,16,0,0,0-16-16H168V40a8,8,0,0,0-16,0V56H136V40a8,8,0,0,0-16,0V56H104V40a8,8,0,0,0-16,0V56H72A16,16,0,0,0,56,72V88H40a8,8,0,0,0,0,16H56v32H40a8,8,0,0,0,0,16H56v16a16,16,0,0,0,16,16H88v16a8,8,0,0,0,16,0V200h32v16a8,8,0,0,0,16,0V200h16a16,16,0,0,0,16-16V168h16a8,8,0,0,0,0-16H200V120h16a8,8,0,0,0,0-16ZM184,184H72V72H184ZM144,104v48a8,8,0,0,1-8,8H120a8,8,0,0,1-8-8V104a8,8,0,0,1,8-8h16A8,8,0,0,1,144,104Zm-16,40V112H128v32Z"/></svg> }
}

#[component]
pub fn Memory(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M224,72v88a16,16,0,0,1-16,16H48a16,16,0,0,1-16-16V72A16,16,0,0,1,48,56H208A16,16,0,0,1,224,72Zm-16,0H48v88H208ZM72,112a8,8,0,0,0-8,8v16a8,8,0,0,0,16,0V120A8,8,0,0,0,72,112Zm40,0a8,8,0,0,0-8,8v16a8,8,0,0,0,16,0V120A8,8,0,0,0,112,112Zm40,0a8,8,0,0,0-8,8v16a8,8,0,0,0,16,0V120A8,8,0,0,0,152,112ZM72,184H88v16a8,8,0,0,0,16,0V184h16v16a8,8,0,0,0,16,0V184h16v16a8,8,0,0,0,16,0V184h16v16a8,8,0,0,0,16,0V184h8a8,8,0,0,0,0-16H72a8,8,0,0,0,0,16Z"/></svg> }
}

#[component]
pub fn WarningCircle(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24Zm0,192a88,88,0,1,1,88-88A88.1,88.1,0,0,1,128,216ZM120,80v48a8,8,0,0,0,16,0V80a8,8,0,0,0-16,0Zm20,88a12,12,0,1,1-12-12A12,12,0,0,1,140,168Z"/></svg> }
}

#[component]
pub fn Wrench(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M224,67.31V32a8,8,0,0,0-8-8H180.69A15.86,15.86,0,0,0,169.37,28.69L114.75,83.31A72,72,0,1,0,172.69,141.25l54.62-54.62A15.86,15.86,0,0,0,232,75.31C232,72.41,228.64,68.91,224,67.31ZM96,208A56,56,0,1,1,152,152v6.63L114.34,196.34A8,8,0,0,0,125.66,207.66l44.41-44.41A56,56,0,0,1,96,208ZM216,72.69l-52.69,52.68L130.63,92.69A8,8,0,0,0,119.31,104l32.69,32.69L199.31,84V40h44v19.31C243.61,64.27,222.9,72.58,216,72.69Z"/></svg> }
}

#[component]
pub fn Brain(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M216,110.87a64.06,64.06,0,0,0-58.87-62.77V40a8,8,0,0,0-16,0V48.1a64.06,64.06,0,0,0-58.87,62.77A64,64,0,0,0,40,170.87,64.12,64.12,0,0,0,82.26,220a63.92,63.92,0,0,0,91.48,0A64.12,64.12,0,0,0,216,170.87,64,64,0,0,0,216,110.87ZM128,64a48,48,0,0,1,47.8,44h-95.6A48,48,0,0,1,128,64Zm-36.46,144A48,48,0,0,1,56,170.87a48.05,48.05,0,0,1,41.93-47.53l5.07,70.66ZM128,208.53l-4.78-66.53h9.56ZM159.53,208l5.07-70.66a48.05,48.05,0,0,1,41.93,47.53A48,48,0,0,1,159.53,208Z"/></svg> }
}

#[component]
pub fn Stack(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M240,111,133.33,164.17a15.93,15.93,0,0,1-14.66,0L12,111a16,16,0,0,1,0-28.28L118.67,29.83a15.93,15.93,0,0,1,14.66,0L240,82.72A16,16,0,0,1,240,111ZM45,96.85,128,138.35l83-41.5-83-41.5ZM240,143a16,16,0,0,1-16,16H32a16,16,0,0,1,0-32H224A16,16,0,0,1,240,143Zm0,48a16,16,0,0,1-16,16H32a16,16,0,0,1,0-32H224A16,16,0,0,1,240,188Z"/></svg> }
}

#[component]
pub fn Plus(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M224,128a8,8,0,0,1-8,8H136v80a8,8,0,0,1-16,0V136H40a8,8,0,0,1,0-16h80V40a8,8,0,0,1,16,0v80h80A8,8,0,0,1,224,128Z"/></svg> }
}

#[component]
pub fn Trash(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M216,48H176V40a24,24,0,0,0-24-24H104A24,24,0,0,0,80,40v8H40a8,8,0,0,0,0,16h8V208a24,24,0,0,0,24,24H184a24,24,0,0,0,24-24V64h8a8,8,0,0,0,0-16ZM96,40a8,8,0,0,1,8-8h48a8,8,0,0,1,8,8v8H96Zm96,168a8,8,0,0,1-8,8H72a8,8,0,0,1-8-8V64H192ZM112,96v80a8,8,0,0,1-16,0V96a8,8,0,0,1,16,0Zm48,0v80a8,8,0,0,1-16,0V96a8,8,0,0,1,16,0Z"/></svg> }
}

#[component]
pub fn Terminal(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M123.66,133.66a8,8,0,0,1-11.32,0l-40-40a8,8,0,0,1,11.32-11.32L118,116.69l34.34-34.35a8,8,0,0,1,11.32,11.32l-40,40ZM216,40H40A16,16,0,0,0,24,56V200a16,16,0,0,0,16,16H216a16,16,0,0,0,16-16V56A16,16,0,0,0,216,40Zm0,160H40V56H216V200Zm-32-48H128a8,8,0,0,0,0,16h56a8,8,0,0,0,0-16Z"/></svg> }
}

#[component]
pub fn Play(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M228.23,123.07,92.25,37.05A16,16,0,0,0,68,50.48V205.52a16,16,0,0,0,24.25,13.43l135.98-86.03A16,16,0,0,0,228.23,123.07ZM84,205.52V50.48L220,128Z"/></svg> }
}

#[component]
pub fn Stop(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M192,40H64A16,16,0,0,0,48,56V192a16,16,0,0,0,16,16H192a16,16,0,0,0,16-16V56A16,16,0,0,0,192,40Zm0,152H64V56H192V192Z"/></svg> }
}

#[component]
pub fn ArrowCounterClockwise(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M224,128a96,96,0,0,1-94.72,96H128a95.38,95.38,0,0,1-67.88-28.12l-1.47-1.47A96,96,0,0,1,128,32h.72a95.38,95.38,0,0,1,67.88,28.12L211.31,74.74V48a8,8,0,0,1,16,0v40a8,8,0,0,1-8,8H179.31a8,8,0,0,1,0-16h20.69l-14.73-14.74A79.4,79.4,0,0,0,128.72,48H128a80,80,0,1,0,80,80,8,8,0,0,1,16,0Z"/></svg> }
}

#[component]
pub fn Gear(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M128,80a48,48,0,1,0,48,48A48.05,48.05,0,0,0,128,80Zm0,80a32,32,0,1,1,32-32A32,32,0,0,1,128,160Zm88,40a24,24,0,0,1-24,24H72a24,24,0,0,1-24-24V191a23.9,23.9,0,0,1,4.4-14l27.1-40.7a8,8,0,0,1,13.3,0l12,18A16,16,0,0,0,118,161.7l1.7-8.5a8,8,0,0,1,15.7,3.1L128,193a8,8,0,0,1,1.9-3.9l11.6,17.4A23.9,23.9,0,0,1,146,220h38a8,8,0,0,1,0,16H146a23.9,23.9,0,0,1-13.3-4.1L128,225.1l-4.7,6.8A23.9,23.9,0,0,1,110,236H72a40,40,0,0,1-40-40V160a24,24,0,0,1,4.4-14l30.2-45.3a24,24,0,0,1,39.9,0L135.2,143a40.1,40.1,0,0,1,73.5-19L223.4,146l30.2,45.3A24,24,0,0,1,256,205.3V240a16,16,0,0,1-16,16H208a8,8,0,0,1,0-16h32V205.3a8,8,0,0,0-1.5-4.7l-30.2-45.3a8,8,0,0,0-13.3,0l-12,18a16,16,0,0,0,13.3,25l13.7,20.6A24,24,0,0,1,216,211a7.92,7.92,0,0,1-4.4,7.1l-25,12.5a8,8,0,1,1-7.2-14.3l19.5-9.7a8,8,0,0,0,1.5-4.7v-25a8,8,0,0,0-13.3-6l-11.6,17.4a23.9,23.9,0,0,1-13.3,13.3l-12.5,25a8,8,0,0,1-14.3-7.2L128,211l4.7-23.5a16,16,0,0,0-1.8-12.1l-14.1-21.2a8,8,0,0,0-13.3,0L73.3,199.5A24,24,0,0,1,53.4,211l-25,12.5a8,8,0,0,1-7.2-14.3l19.5-9.7a8,8,0,0,0,1.5-4.7v-25a8,8,0,0,0-1.5-4.7l-30.2-45.3a8,8,0,0,0-13.3,0l-12,18A16,16,0,0,0,0,161.7l1.7,8.5a8,8,0,0,1-15.7,3.1L0,193a8,8,0,0,1-1.9-3.9l-11.6,17.4A23.9,23.9,0,0,1-24,220H14a8,8,0,0,1,0,16H-24a23.9,23.9,0,0,1-13.3-4.1l-4.7-6.8l-4.7,6.8A23.9,23.9,0,0,1-60,236h-38a40,40,0,0,1-40-40V160a24,24,0,0,1,4.4-14l30.2-45.3a24,24,0,0,1,39.9,0L0,143a40.1,40.1,0,0,1,73.5,19V240a16,16,0,0,1-16,16H16a8,8,0,0,1,0-16h41.7V172.9a23.9,23.9,0,0,1-4.4-14l-27.1-40.7a8,8,0,0,1,13.3,0l12,18A16,16,0,0,0,65.3,161.7l1.7-8.5a8,8,0,0,1,15.7,3.1L64,193a8,8,0,0,1,1.9-3.9l11.6,17.4A23.9,23.9,0,0,1,82,220h38a8,8,0,0,1,0,16H82a23.9,23.9,0,0,1-13.3-4.1l-4.7-6.8l-4.7,6.8A23.9,23.9,0,0,1,46,236H8a40,40,0,0,1-40-40V160a24,24,0,0,1,4.4-14l30.2-45.3a24,24,0,0,1,39.9,0L8,143a40.1,40.1,0,0,1,73.5,19V240a16,16,0,0,1-16,16H24a8,8,0,0,1,0-16h32V205.3a8,8,0,0,0-1.5-4.7l-30.2-45.3a8,8,0,0,0-13.3,0l-12,18a16,16,0,0,0,13.3,25l13.7,20.6A24,24,0,0,1,52,211a7.92,7.92,0,0,1-4.4,7.1l-25,12.5a8,8,0,1,1-7.2-14.3l19.5-9.7a8,8,0,0,0,1.5-4.7v-25a8,8,0,0,0-13.3-6l-11.6,17.4a23.9,23.9,0,0,1-13.3,13.3l-12.5,25a8,8,0,0,1-14.3-7.2L0,211l4.7-23.5a16,16,0,0,0-1.8-12.1l-14.1-21.2a8,8,0,0,0-13.3,0L-54.7,199.5A24,24,0,0,1-74.6,211l-25,12.5a8,8,0,0,1-7.2-14.3l19.5-9.7a8,8,0,0,0,1.5-4.7v-25a8,8,0,0,0-1.5-4.7l-30.2-45.3a8,8,0,0,0-13.3,0l-12,18A16,16,0,0,0-128,161.7l1.7,8.5a8,8,0,0,1-15.7,3.1L-128,193a8,8,0,0,1-1.9-3.9l-11.6,17.4A23.9,23.9,0,0,1-152,220h14a8,8,0,0,1,0,16H-152a23.9,23.9,0,0,1-13.3-4.1l-4.7-6.8V160A24,24,0,0,1-152,146l30.2-45.3a24,24,0,0,1,39.9,0L-93.5,143a40.1,40.1,0,0,1,73.5,19V240a16,16,0,0,1-16,16H-80a8,8,0,0,1,0-16h32V205.3a8,8,0,0,0-1.5-4.7l-30.2-45.3a8,8,0,0,0-13.3,0l-12,18a16,16,0,0,0,13.3,25l13.7,20.6A24,24,0,0,1-52,211a7.92,7.92,0,0,1-4.4,7.1l-25,12.5a8,8,0,1,1-7.2-14.3l19.5-9.7a8,8,0,0,0,1.5-4.7v-25a8,8,0,0,0-13.3-6l-11.6,17.4A23.9,23.9,0,0,1-93.3,181.3L-105.8,206.3a8,8,0,0,1-14.3-7.2L-115.4,188.4a16,16,0,0,0-1.8-12.1l-14.1-21.2a8,8,0,0,0-13.3,0L-181.3,197.8A24,24,0,0,1-201.2,209.3l-25,12.5a8,8,0,0,1-7.2-14.3l19.5-9.7a8,8,0,0,0,1.5-4.7V168a8,8,0,0,1,16,0V193.1a8,8,0,0,0,1.5,4.7l27.1,40.6A8,8,0,0,0-181.3,251.3l12-18a16,16,0,0,0-13.3-25l-27.1-40.6A24,24,0,0,1-214.1,153.7L-183.9,108.4a24,24,0,0,1,39.9,0L-113.8,153.7A23.9,23.9,0,0,1-109.4,167.7V251.3a16,16,0,0,1-16,16H-160a8,8,0,0,1,0-16H-125.4V167.7a8,8,0,0,0-1.5-4.7L-154,122.3a8,8,0,0,0-13.3,0l-30.2,45.3a8,8,0,0,0,0,8.9l30.2,45.3A8,8,0,0,0-160,226.5V251.3a16,16,0,0,1-16,16H-208a8,8,0,0,1,0-16H-176V226.5a24,24,0,0,1,4.4-14L-144.5,171.8a56.1,56.1,0,0,0,0-67.6L-174.7,58.9A24,24,0,0,1-153,30.3H-103a24,24,0,0,1,21.7,28.6L-93.8,111.9A39.9,39.9,0,0,0-128,152v6.63L-165.7,196.3a8,8,0,1,0,11.3,11.3l44.4-44.4A56,56,0,0,1-128,208Z"/></svg> }
}
#[component]
pub fn CloudArrowUp(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M160,152a8,8,0,0,1-8,8H136v40a8,8,0,0,1-16,0V160H104a8,8,0,0,1,0-16h48A8,8,0,0,1,160,152Zm79,8a88,88,0,1,1-153.29-60.67A80,80,0,1,1,223,73.49,56,56,0,0,1,239,160Zm-16,0a40,40,0,0,0-20.12-34.61,8,8,0,0,1-3.88-6.89c.1-1.63.15-3.23.15-4.83a64,64,0,1,0-128,0c0,1.6.05,3.2.15,4.8a8,8,0,0,1-3.88,6.89,72,72,0,1,0,123.6,54.64,8,8,0,0,1,16,0c0,.1,0,.2,0,.3A39.75,39.75,0,0,0,223,160Z"/></svg> }
}

#[component]
pub fn Globe(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24ZM101.63,168h52.74C149,186.34,140,202.87,128,213.65,116,202.87,107.05,186.34,101.63,168ZM98,152a145.72,145.72,0,0,1,0-48h60a145.72,145.72,0,0,1,0,48ZM40,128a87.61,87.61,0,0,1,3.33-24H81.79a161.79,161.79,0,0,0,0,48H43.33A87.61,87.61,0,0,1,40,128Zm114.37-40H101.63C107.05,69.66,116,53.13,128,42.35,140,53.13,149,69.66,154.37,88Zm19.84,16h38.46a88.15,88.15,0,0,1,0,48H174.21a161.79,161.79,0,0,0,0-48Zm32.16-16H170.94a142.39,142.39,0,0,0-20.26-45A88.37,88.37,0,0,1,206.37,88ZM105.32,43A142.39,142.39,0,0,0,85.06,88H49.63A88.37,88.37,0,0,1,105.32,43ZM49.63,168H85.06a142.39,142.39,0,0,0,20.26,45A88.37,88.37,0,0,1,49.63,168Zm101.05,45a142.39,142.39,0,0,0,20.26-45h35.43A88.37,88.37,0,0,1,150.68,213Z"/></svg> }
}

#[component]
pub fn Lock(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M208,80H176V56a48,48,0,0,0-96,0V80H48A16,16,0,0,0,32,96V208a16,16,0,0,0,16,16H208a16,16,0,0,0,16-16V96A16,16,0,0,0,208,80ZM96,56a32,32,0,0,1,64,0V80H96ZM208,208H48V96H208V208Zm-68-56a12,12,0,1,1-12-12A12,12,0,0,1,140,152Z"/></svg> }
}

#[component]
pub fn Folder(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M216,72H130.67L102.93,44.27A15.89,15.89,0,0,0,91.66,39.61H40A16,16,0,0,0,24,55.61V200.39A16,16,0,0,0,40,216.39H216a16,16,0,0,0,16-16V88A16,16,0,0,0,216,72Zm0,128.39H40V55.61H91.66l27.73,27.74A15.89,15.89,0,0,0,130.67,88H216Z"/></svg> }
}

#[component]
pub fn File(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M213.66,82.34l-56-56A8,8,0,0,0,152,24H56A16,16,0,0,0,40,40V216a16,16,0,0,0,16,16H200a16,16,0,0,0,16-16V88A8,8,0,0,0,213.66,82.34ZM160,51.31,188.69,80H160ZM200,216H56V40h88V88a8,8,0,0,0,8,8h48V216Z"/></svg> }
}

#[component]
pub fn Upload(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M221.66,133.66a8,8,0,0,1-11.32,0L136,59.31V160a8,8,0,0,1-16,0V59.31L45.66,133.66a8,8,0,0,1-11.32-11.32l88-88a8,8,0,0,1,11.32,0l88,88A8,8,0,0,1,221.66,133.66ZM216,200H40a8,8,0,0,0,0,16H216a8,8,0,0,0,0-16Z"/></svg> }
}

#[component]
pub fn Download(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M213.66,122.34a8,8,0,0,0-11.32,0L136,196.69V32a8,8,0,0,0-16,0V196.69L53.66,122.34a8,8,0,0,0-11.32,11.32l88,88a8,8,0,0,0,11.32,0l88-88A8,8,0,0,0,213.66,122.34ZM216,200H40a8,8,0,0,0,0,16H216a8,8,0,0,0,0-16Z"/></svg> }
}

#[component]
pub fn Edit(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M216,16a32,32,0,0,0-32,32v8h-8V40a8,8,0,0,0-16,0v16H144a8,8,0,0,0,0,16h16v16h-8a8,8,0,0,0,0,16h8v8a32,32,0,0,0,32,32h8v8a8,8,0,0,0,16,0V144h16a8,8,0,0,0,0-16H208V112h8a8,8,0,0,0,0-16H208V88h8a32,32,0,0,0,32-32V40A32,32,0,0,0,216,16Zm16,40a16,16,0,0,1-16,16H200V48h16a16,16,0,0,1,16,16Z"/></svg> }
}

#[component]
pub fn X(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M205.66,194.34a8,8,0,0,1-11.32,11.32L128,139.31,61.66,205.66a8,8,0,0,1-11.32-11.32L116.69,128,50.34,61.66A8,8,0,0,1,61.66,50.34L128,116.69l66.34-66.35a8,8,0,0,1,11.32,11.32L139.31,128Z"/></svg> }
}

#[component]
pub fn Search(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M229.66,218.34l-50.07-50.06a88.11,88.11,0,1,0-11.31,11.31l50.06,50.07a8,8,0,0,0,11.32-11.32ZM40,112a72,72,0,1,1,72,72A72.08,72.08,0,0,1,40,112Z"/></svg> }
}

#[component]
pub fn ChevronRight(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M181.66,133.66l-80,80a8,8,0,0,1-11.32-11.32L164.69,128,90.34,53.66a8,8,0,0,1,11.32-11.32l80,80A8,8,0,0,1,181.66,133.66Z"/></svg> }
}

#[component]
pub fn Copy(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M216,32H88a16,16,0,0,0-16,16V72H48A16,16,0,0,0,32,88V216a16,16,0,0,0,16,16H176a16,16,0,0,0,16-16V176h24a16,16,0,0,0,16-16V48A16,16,0,0,0,216,32ZM176,216H48V88H176ZM216,160H192V88a16,16,0,0,0-16-16H88V48H216Z"/></svg> }
}

#[component]
pub fn Scissors(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M228.3,136.6a8,8,0,0,0-1.89-10.3c-1.39-1.12-23.77-18.3-90.41-18.3s-89,17.18-90.41,18.3a8,8,0,0,0,1.89,14.3C48.77,141,71.15,158.18,137.79,158.18s89-17.18,90.41-18.3A8,8,0,0,0,228.3,136.6ZM137.79,142.18c-44.57,0-64.88-9.45-73.47-14.18,8.59-4.73,28.9-14.18,73.47-14.18s64.88,9.45,73.47,14.18C202.67,132.73,182.36,142.18,137.79,142.18Z"/></svg> }
}

#[component]
pub fn Clipboard(#[prop(into, optional, default=String::new())] class: String) -> impl IntoView {
    view! { <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor" class=class><path d="M200,64h-8V56a40,40,0,0,0-40-40h-48A40,40,0,0,0,64,56v8h-8A16,16,0,0,0,40,80V216a16,16,0,0,0,16,16H200a16,16,0,0,0,16-16V80A16,16,0,0,0,200,64ZM80,56a24,24,0,0,1,24-24h48a24,24,0,0,1,24,24v8H80ZM200,216H56V80h144V216Zm-68-56a12,12,0,1,1-12-12A12,12,0,0,1,140,152Z"/></svg> }
}
