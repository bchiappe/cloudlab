use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VM {
    pub id: String,
    pub host_id: String,
    pub host_name: String,
    pub name: String,
    pub cpu: i32,
    pub memory_mb: i32,
    pub status: String,
    pub os_type: String,
    pub disk_volume_id: Option<String>,
    pub iso_volume_id: Option<String>,
    pub boot_device: String,
    pub mac_address: Option<String>,
    pub vnc_port: Option<i32>,
    pub vnc_token: Option<String>,
}

#[cfg(feature = "ssr")]
use duckdb::params;

#[cfg(feature = "ssr")]
use crate::auth::srv_err;

// ─── List ────────────────────────────────────────────────────────────────────

#[server(ListVMs, "/api")]
pub async fn list_vms() -> Result<Vec<VM>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let vms = tokio::task::spawn_blocking(move || -> Result<Vec<VM>, ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let mut stmt = conn
                .prepare(
                    "SELECT v.id, v.host_id, h.name as host_name, v.name, v.cpu, v.memory_mb, v.status, v.os_type, \
                     v.disk_volume_id, v.iso_volume_id, v.boot_device, v.mac_address, v.vnc_port, v.vnc_token \
                     FROM vms v \
                     LEFT JOIN hosts h ON v.host_id = h.id \
                     ORDER BY v.name ASC;",
                )
                .map_err(srv_err)?;
            let mut rows = stmt.query(params![]).map_err(srv_err)?;
            let mut vms = Vec::new();
            while let Some(row) = rows.next().map_err(srv_err)? {
                vms.push(VM {
                    id: row.get::<_, String>(0).map_err(srv_err)?,
                    host_id: row.get::<_, String>(1).map_err(srv_err).unwrap_or_default(),
                    host_name: row.get::<_, String>(2).map_err(srv_err).unwrap_or_else(|_| "Unknown".into()),
                    name: row.get::<_, String>(3).map_err(srv_err)?,
                    cpu: row.get::<_, i32>(4).map_err(srv_err)?,
                    memory_mb: row.get::<_, i32>(5).map_err(srv_err)?,
                    status: row.get::<_, String>(6).map_err(srv_err)?,
                    os_type: row.get::<_, String>(7).map_err(srv_err)?,
                    disk_volume_id: row.get::<_, Option<String>>(8).map_err(srv_err)?,
                    iso_volume_id: row.get::<_, Option<String>>(9).map_err(srv_err)?,
                    boot_device: row.get::<_, String>(10).map_err(srv_err)?,
                    mac_address: row.get::<_, Option<String>>(11).map_err(srv_err)?,
                    vnc_port: row.get::<_, Option<i32>>(12).map_err(srv_err)?,
                    vnc_token: row.get::<_, Option<String>>(13).map_err(srv_err)?,
                });
            }
            Ok(vms)
        })
        .await
        .map_err(srv_err)??;

        Ok(vms)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Create ──────────────────────────────────────────────────────────────────

#[server(CreateVM, "/api")]
pub async fn create_vm(
    host_id: String,
    name: String,
    cpu: i32,
    memory_mb: i32,
    os_type: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO vms (id, host_id, name, cpu, memory_mb, status, os_type) \
                 VALUES (?, ?, ?, ?, ?, 'stopped', ?);",
                params![id, host_id, name, cpu, memory_mb, os_type],
            )
            .map_err(srv_err)?;
            Ok(())
        })
        .await
        .map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Update ──────────────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn generate_vm_xml(vm: &VM, bridge: &str, disk_path: &str, iso_path: Option<&str>) -> String {
    let boot_dev = if vm.boot_device == "cdrom" { "cdrom" } else { "hd" };
    let iso_xml = if let Some(path) = iso_path {
        format!(
            r#"<disk type='file' device='cdrom'>
      <driver name='qemu' type='raw'/>
      <source file='{}'/>
      <target dev='sda' bus='sata'/>
      <readonly/>
    </disk>"#,
            path
        )
    } else {
        "".to_string()
    };

    let mac_xml = if let Some(mac) = &vm.mac_address {
        format!("<mac address='{}'/>", mac)
    } else {
        "".to_string()
    };

    format!(
        r#"<domain type='kvm'>
  <name>{}</name>
  <uuid>{}</uuid>
  <memory unit='MiB'>{}</memory>
  <vcpu placement='static'>{}</vcpu>
  <os>
    <type arch='x86_64' machine='pc-q35-4.2'>hvm</type>
    <boot dev='{}'/>
  </os>
  <features>
    <acpi/><apic/><pae/>
  </features>
  <cpu mode='host-passthrough' check='none'/>
  <clock offset='utc'/>
  <on_poweroff>destroy</on_poweroff>
  <on_reboot>restart</on_reboot>
  <on_crash>destroy</on_crash>
  <devices>
    <emulator>/usr/bin/qemu-system-x86_64</emulator>
    <disk type='block' device='disk'>
      <driver name='qemu' type='raw' cache='none' io='native'/>
      <source dev='{}'/>
      <target dev='vda' bus='virtio'/>
    </disk>
    {}
    <interface type='bridge'>
      {}
      <source bridge='{}'/>
      <model type='virtio'/>
    </interface>
    <graphics type='vnc' port='-1' autoport='yes' listen='0.0.0.0'>
      <listen type='address' address='0.0.0.0'/>
    </graphics>
    <video>
      <model type='virtio' vram='65536' heads='1' primary='yes'/>
    </video>
    <console type='pty'>
      <target type='serial' port='0'/>
    </console>
  </devices>
</domain>"#,
        vm.name, vm.id, vm.memory_mb, vm.cpu, boot_dev, disk_path, iso_xml, mac_xml, bridge
    )
}

#[cfg(feature = "ssr")]
async fn get_vm_and_host(pool: r2d2::Pool<duckdb::DuckdbConnectionManager>, id: String) -> Result<(VM, crate::hosts::Host), ServerFnError> {
    tokio::task::spawn_blocking(move || -> Result<(VM, crate::hosts::Host), ServerFnError> {
        let conn = pool.get().map_err(srv_err)?;
        let mut stmt = conn.prepare("SELECT v.id, v.host_id, h.name, v.name, v.cpu, v.memory_mb, v.status, v.os_type, v.disk_volume_id, v.iso_volume_id, v.boot_device, v.mac_address, v.vnc_port, v.vnc_token, h.address, h.port, h.username, h.password, h.ssh_key, h.ssh_passphrase FROM vms v JOIN hosts h ON v.host_id = h.id WHERE v.id = ?").map_err(srv_err)?;
        
        let (vm, host_addr, host_port, host_user, host_pass, host_ssh_key, host_ssh_pp) = stmt.query_row(params![id], |row| {
            Ok((
                VM {
                    id: row.get(0)?,
                    host_id: row.get(1)?,
                    host_name: row.get(2)?,
                    name: row.get(3)?,
                    cpu: row.get(4)?,
                    memory_mb: row.get(5)?,
                    status: row.get(6)?,
                    os_type: row.get(7)?,
                    disk_volume_id: row.get(8)?,
                    iso_volume_id: row.get(9)?,
                    boot_device: row.get(10)?,
                    mac_address: row.get(11)?,
                    vnc_port: row.get(12)?,
                    vnc_token: row.get(13)?,
                },
                row.get::<_, String>(14)?,
                row.get::<_, i32>(15)?,
                row.get::<_, String>(16)?,
                row.get::<_, Option<String>>(17)?,
                row.get::<_, Option<String>>(18)?,
                row.get::<_, Option<String>>(19)?,
            ))
        }).map_err(srv_err)?;

        let host = crate::hosts::Host {
            id: vm.host_id.clone(),
            name: vm.host_name.clone(),
            address: host_addr,
            port: host_port,
            username: host_user,
            password: host_pass,
            ssh_key: host_ssh_key,
            ssh_passphrase: host_ssh_pp,
            ..Default::default()
        };

        Ok((vm, host))
    }).await.map_err(srv_err)?
}

#[server(DeployVM, "/api")]
pub async fn deploy_vm(id: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        // 1. Fetch VM and Host details
        let (vm, host) = get_vm_and_host(pool.clone(), id.clone()).await?;

        let job_id = crate::jobs::create_job(pool.clone(), format!("Deploy VM: {}", vm.name)).await?;
        let pool_task = pool.clone();
        let job_id_task = job_id.clone();

        tokio::spawn(async move {
            let _ = crate::jobs::update_job(pool_task.clone(), job_id_task.clone(), "running".into(), 10).await;
            
            let res: Result<(), String> = (async {
                // 2. Ensure Storage
                let vol_name = vm.disk_volume_id.clone().unwrap_or_else(|| format!("vm-{}-disk", vm.name));
                let _ = crate::jobs::add_job_log(pool_task.clone(), job_id_task.clone(), format!("Ensuring Linstor volume '{}'...", vol_name)).await;
                
                let controller = crate::hosts::get_controller_host(pool_task.clone()).await.map_err(|e| e.to_string())?
                    .ok_or_else(|| "No Linstor controller found".to_string())?;
                let linstor = crate::storage::linstor::LinstorClient::new(&controller.address);
                
                // Fix: passing u64 for size_gb
                linstor.create_volume(pool_task.clone(), job_id_task.clone(), &vol_name, 20, 1).await?;
                
                // 3. Connect to Host and Deploy
                let _ = crate::jobs::add_job_log(pool_task.clone(), job_id_task.clone(), format!("Connecting to host {}...", host.address)).await;
                let sess = crate::hosts::establish_ssh_session(&host).map_err(|e| e.to_string())?;
                
                // 4. Detect Bridge
                let (_, stdout, _) = crate::ssh::run_remote_script(&sess, "brctl show | awk 'NR>1 {print $1}' | grep -E 'br0|virbr0' | head -n 1", None).map_err(|e| e.to_string())?;
                let bridge = if stdout.trim().is_empty() { "virbr0" } else { stdout.trim() };
                
                // 5. Generate and Define XML
                let disk_path = format!("/dev/linstor/cloudlab_pool/{}", vol_name);
                let xml = generate_vm_xml(&vm, bridge, &disk_path, None);
                let xml_escaped = xml.replace("'", "'\\''");
                
                let define_script = format!("echo '{}' > /tmp/{}.xml && virsh define /tmp/{}.xml && virsh start {}", xml_escaped, vm.id, vm.id, vm.name);
                let (status, out, err) = crate::ssh::run_remote_script(&sess, &define_script, host.password.as_deref()).map_err(|e| e.to_string())?;
                
                if status != 0 {
                    return Err(format!("Libvirt Error: {}\n{}", out, err));
                }

                // 6. Update DB with real state
                let p2 = pool_task.clone();
                let vid = vm.id.clone();
                let vname = vol_name.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    let conn = p2.get().unwrap();
                    let _ = conn.execute("UPDATE vms SET disk_volume_id = ?, status = 'running' WHERE id = ?", duckdb::params![vname, vid]);
                }).await;

                Ok(())
            }).await;

            match res {
                Ok(_) => {
                    let _ = crate::jobs::update_job(pool_task, job_id_task, "completed".into(), 100).await;
                }
                Err(e) => {
                    let _ = crate::jobs::add_job_log(pool_task.clone(), job_id_task.clone(), format!("Error: {}", e)).await;
                    let _ = crate::jobs::update_job(pool_task, job_id_task, "failed".into(), 0).await;
                }
            }
        });

        Ok(job_id)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(UpdateVM, "/api")]
pub async fn update_vm(
    id: String,
    host_id: String,
    name: String,
    cpu: i32,
    memory_mb: i32,
    os_type: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        // 1. Update DB
        tokio::task::spawn_blocking({
            let pool = pool.clone();
            let id = id.clone();
            move || -> Result<(), ServerFnError> {
                let conn = pool.get().map_err(srv_err)?;
                conn.execute(
                    "UPDATE vms SET host_id=?, name=?, cpu=?, memory_mb=?, os_type=? \
                     WHERE id=?;",
                    params![host_id, name, cpu, memory_mb, os_type, id],
                )
                .map_err(srv_err)?;
                Ok(())
            }
        })
        .await
        .map_err(srv_err)??;

        // 2. Refresh host config if VM is already defined
        let (vm, host) = get_vm_and_host(pool.clone(), id.clone()).await?;

        if let Some(vol_name) = &vm.disk_volume_id {
            let sess = crate::hosts::establish_ssh_session(&host).map_err(srv_err)?;
            
            // Detect Bridge
            let (_, stdout, _) = crate::ssh::run_remote_script(&sess, "brctl show | awk 'NR>1 {{print $1}}' | grep -E 'br0|virbr0' | head -n 1", None).map_err(srv_err)?;
            let bridge = if stdout.trim().is_empty() { "virbr0" } else { stdout.trim() };
            
            let disk_path = format!("/dev/linstor/cloudlab_pool/{}", vol_name);
            let xml = generate_vm_xml(&vm, bridge, &disk_path, None);
            let xml_escaped = xml.replace("'", "'\\''");
            
            let define_script = format!("echo '{}' > /tmp/{}.xml && virsh define /tmp/{}.xml", xml_escaped, vm.id, vm.id);
            let (status, _, err) = crate::ssh::run_remote_script(&sess, &define_script, host.password.as_deref()).map_err(srv_err)?;
            
            if status != 0 {
                return Err(srv_err(format!("Update failed: {}", err)));
            }
        }
        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Delete ──────────────────────────────────────────────────────────────────

#[server(DeleteVM, "/api")]
pub async fn delete_vm(id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        tokio::task::spawn_blocking(move || -> Result<(), ServerFnError> {
            let conn = pool.get().map_err(srv_err)?;
            conn.execute("DELETE FROM vms WHERE id=?;", params![id])
                .map_err(srv_err)?;
            Ok(())
        })
        .await
        .map_err(srv_err)??;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ─── Toggle Power ────────────────────────────────────────────────────────────

#[server(ToggleVMPower, "/api")]
pub async fn toggle_vm_power(id: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let (vm, host) = get_vm_and_host(pool.clone(), id.clone()).await?;
        let sess = crate::hosts::establish_ssh_session(&host).map_err(srv_err)?;

        let current_status = vm.status.clone();
        let next_status = if current_status == "running" {
            // Power off
            let (status, _, err) = crate::ssh::run_remote_script(&sess, &format!("virsh destroy {}", vm.name), host.password.as_deref()).map_err(srv_err)?;
            if status != 0 && !err.contains("domain is not running") {
                return Err(srv_err(format!("Stop failed: {}", err)));
            }
            "stopped"
        } else {
            // Power on
            let (status, _, err) = crate::ssh::run_remote_script(&sess, &format!("virsh start {}", vm.name), host.password.as_deref()).map_err(srv_err)?;
            if status != 0 {
                return Err(srv_err(format!("Start failed: {}", err)));
            }
            "running"
        }.to_string();

        // Update DB
        tokio::task::spawn_blocking({
            let pool = pool.clone();
            let id = id.clone();
            let next_status = next_status.clone();
            move || {
                let conn = pool.get().unwrap();
                let _ = conn.execute("UPDATE vms SET status=? WHERE id=?;", params![next_status, id]);
            }
        }).await.map_err(srv_err)?;

        Ok(next_status)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(RebootVM, "/api")]
pub async fn reboot_vm(id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let (vm, host) = get_vm_and_host(pool.clone(), id.clone()).await?;
        let sess = crate::hosts::establish_ssh_session(&host).map_err(srv_err)?;

        let (status, _, err) = crate::ssh::run_remote_script(&sess, &format!("virsh reboot {}", vm.name), host.password.as_deref()).map_err(srv_err)?;
        if status != 0 {
            return Err(srv_err(format!("Reboot failed: {}", err)));
        }

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(MountISO, "/api")]
pub async fn mount_iso(id: String, iso_name: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let (vm, host) = get_vm_and_host(pool.clone(), id.clone()).await?;
        let sess = crate::hosts::establish_ssh_session(&host).map_err(srv_err)?;

        let iso_path = format!("/mnt/isos/{}", iso_name);
        
        // Ensure mounted on host
        ensure_iso_volume(&sess, &host.name).await.map_err(srv_err)?;
        
        // Use virsh change-media for live insertion
        let cmd = format!("virsh change-media {} sda {} --insert", vm.name, iso_path);
        let (status, _, err) = crate::ssh::run_remote_script(&sess, &cmd, host.password.as_deref()).map_err(srv_err)?;
        
        if status != 0 {
             return Err(srv_err(format!("Mount failed: {}", err)));
        }

        // Update DB
        tokio::task::spawn_blocking({
            let pool = pool.clone();
            let id = id.clone();
            let iso_name = iso_name.clone();
            move || {
                let conn = pool.get().unwrap();
                let _ = conn.execute("UPDATE vms SET iso_volume_id=? WHERE id=?;", params![iso_name, id]);
            }
        }).await.map_err(srv_err)?;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(UnmountISO, "/api")]
pub async fn unmount_iso(id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let (vm, host) = get_vm_and_host(pool.clone(), id.clone()).await?;
        let sess = crate::hosts::establish_ssh_session(&host).map_err(srv_err)?;

        let cmd = format!("virsh change-media {} sda --eject", vm.name);
        let (status, _, err) = crate::ssh::run_remote_script(&sess, &cmd, host.password.as_deref()).map_err(srv_err)?;
        
        if status != 0 && !err.contains("is already empty") {
             return Err(srv_err(format!("Unmount failed: {}", err)));
        }

        // Update DB
        tokio::task::spawn_blocking({
            let pool = pool.clone();
            let id = id.clone();
            move || {
                let conn = pool.get().unwrap();
                let _ = conn.execute("UPDATE vms SET iso_volume_id=NULL WHERE id=?;", params![id]);
            }
        }).await.map_err(srv_err)?;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(GetVMConsole, "/api")]
pub async fn get_vm_console(id: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let (vm, host) = get_vm_and_host(pool.clone(), id.clone()).await?;
        let sess = crate::hosts::establish_ssh_session(&host).map_err(srv_err)?;

        // 1. Get VNC Port from hypervisor
        let cmd = format!("virsh vncdisplay {}", vm.name);
        let (_, stdout, _) = crate::ssh::run_remote_script(&sess, &cmd, host.password.as_deref()).map_err(srv_err)?;
        
        // Output looks like ":1" or "127.0.0.1:0"
        let display = stdout.trim().trim_start_matches(':');
        let vnc_port = 5900 + display.parse::<i32>().unwrap_or(0);
        
        // 2. Ensure websockify is running on host for this VM
        // We'll use a predictable websockify port: 6000 + display_index
        let ws_port = 6000 + display.parse::<i32>().unwrap_or(0);
        let ws_cmd = format!("pgrep -f 'websockify.*{}' || websockify -D --web /usr/share/novnc/ {} localhost:{}", ws_port, ws_port, vnc_port);
        let _ = crate::ssh::run_remote_script(&sess, &ws_cmd, host.password.as_deref()).map_err(srv_err)?;

        // 3. Construct URL
        let url = format!("http://{}:{}/vnc.html?autoconnect=true", host.address, ws_port);
        
        Ok(url)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(MigrateVM, "/api")]
pub async fn migrate_vm(id: String, target_host_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        // 1. Fetch Source VM and Target Host
        let (vm, source_host) = get_vm_and_host(pool.clone(), id.clone()).await?;
        
        let target_host = tokio::task::spawn_blocking({
            let pool = pool.clone();
            let thid = target_host_id.clone();
            move || -> Result<crate::hosts::Host, ServerFnError> {
                let conn = pool.get().map_err(srv_err)?;
                let mut stmt = conn.prepare("SELECT id, name, address, port, username, password, ssh_key, ssh_passphrase FROM hosts WHERE id = ?").map_err(srv_err)?;
                stmt.query_row(params![thid], |row| {
                    Ok(crate::hosts::Host {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        address: row.get(2)?,
                        port: row.get(3)?,
                        username: row.get(4)?,
                        password: row.get(5)?,
                        ssh_key: row.get(6)?,
                        ssh_passphrase: row.get(7)?,
                        ..Default::default()
                    })
                }).map_err(srv_err)
            }
        }).await.map_err(srv_err)??;

        // 2. Stop and Undefine on Source
        let source_sess = crate::hosts::establish_ssh_session(&source_host).map_err(srv_err)?;
        let _ = crate::ssh::run_remote_script(&source_sess, &format!("virsh destroy {} || true", vm.name), source_host.password.as_deref());
        let _ = crate::ssh::run_remote_script(&source_sess, &format!("virsh undefine {}", vm.name), source_host.password.as_deref());

        // 3. Define and Start on Target
        let target_sess = crate::hosts::establish_ssh_session(&target_host).map_err(srv_err)?;
        
        // Detect Bridge on Target
        let (_, stdout, _) = crate::ssh::run_remote_script(&target_sess, "brctl show | awk 'NR>1 {{print $1}}' | grep -E 'br0|virbr0' | head -n 1", None).map_err(srv_err)?;
        let bridge = if stdout.trim().is_empty() { "virbr0" } else { stdout.trim() };
        
        let vol_name = vm.disk_volume_id.clone().ok_or_else(|| srv_err("VM has no disk volume"))?;
        let disk_path = format!("/dev/linstor/cloudlab_pool/{}", vol_name);
        
        let xml = generate_vm_xml(&vm, bridge, &disk_path, None);
        let xml_escaped = xml.replace("'", "'\\''");
        
        let define_script = format!("echo '{}' > /tmp/{}.xml && virsh define /tmp/{}.xml && virsh start {}", xml_escaped, vm.id, vm.id, vm.name);
        let (status, _, err) = crate::ssh::run_remote_script(&target_sess, &define_script, target_host.password.as_deref()).map_err(srv_err)?;
        
        if status != 0 {
            return Err(srv_err(format!("Migration failed on target: {}", err)));
        }

        // 4. Update DB
        tokio::task::spawn_blocking({
            let pool = pool.clone();
            let id = id.clone();
            let thid = target_host_id.clone();
            move || {
                let conn = pool.get().unwrap();
                let _ = conn.execute("UPDATE vms SET host_id = ?, status = 'running' WHERE id = ?", params![thid, id]);
            }
        }).await.map_err(srv_err)?;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[cfg(feature = "ssr")]
pub async fn ensure_iso_volume(sess: &ssh2::Session, host_name: &str) -> Result<(), String> {
    let script = format!(
        "set -ex\n\
         if ! mountpoint -q /mnt/isos; then\n\
           sudo linstor resource-definition create cloudlab-isos || true\n\
           if ! sudo linstor volume-definition list -r cloudlab-isos | grep -q ' 0 '; then\n\
             sudo linstor volume-definition create cloudlab-isos 50G\n\
           fi\n\
           if ! sudo linstor resource list -n {0} -r cloudlab-isos | grep -q 'cloudlab-isos'; then\n\
             sudo linstor resource create {0} cloudlab-isos --storage-pool cloudlab_pool || \\\n\
             sudo linstor resource create {0} cloudlab-isos --storage-pool cloudlab_pool --layer-list STORAGE\n\
           fi\n\
           DEVICE=''\n\
           for i in $(seq 1 10); do\n\
             DEVICE=$(sudo linstor --no-utf8 volume list -n {0} -r cloudlab-isos | grep '/dev/' | grep -o '/dev/[^ ]*') || true\n\
             if [[ -b \"$DEVICE\" ]]; then break; fi\n\
             sleep 1\n\
           done\n\
           if [[ ! -b \"$DEVICE\" ]]; then echo 'Device not found'; exit 1; fi\n\
           if ! sudo blkid \"$DEVICE\"; then sudo mkfs.ext4 \"$DEVICE\"; fi\n\
           sudo mkdir -p /mnt/isos\n\
           sudo mount \"$DEVICE\" /mnt/isos\n\
         fi",
        host_name
    );

    let (status, stdout, stderr) = crate::ssh::run_remote_script(sess, &script, None).map_err(|e| e.to_string())?;
    if status != 0 {
        return Err(format!("Failed to ensure ISO volume: {}\n{}", stdout, stderr));
    }
    Ok(())
}

#[server(ListISOs, "/api")]
pub async fn list_isos(host_id: String) -> Result<Vec<String>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::auth::require_role(crate::auth::UserRole::Operator).await?;
        use axum::extract::Extension;
        use duckdb::DuckdbConnectionManager;
        use leptos_axum::extract;
        use r2d2::Pool;

        let Extension(pool) = extract::<Extension<Pool<DuckdbConnectionManager>>>()
            .await
            .map_err(|_| srv_err("Failed to extract DB pool"))?;

        let host = tokio::task::spawn_blocking({
            let pool = pool.clone();
            let hid = host_id.clone();
            move || -> Result<crate::hosts::Host, ServerFnError> {
                let conn = pool.get().map_err(srv_err)?;
                let mut stmt = conn.prepare("SELECT id, name, address, port, username, password, ssh_key, ssh_public_key, ssh_passphrase FROM hosts WHERE id = ?").map_err(srv_err)?;
                stmt.query_row(params![hid], |row| {
                    Ok(crate::hosts::Host {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        address: row.get(2)?,
                        port: row.get(3)?,
                        username: row.get(4)?,
                        password: row.get(5)?,
                        ssh_key: row.get(6)?,
                        ssh_public_key: row.get(7)?,
                        ssh_passphrase: row.get(8)?,
                        ..Default::default()
                    })
                }).map_err(srv_err)
            }
        }).await.map_err(srv_err)??;

        let sess = crate::hosts::establish_ssh_session(&host).map_err(srv_err)?;
        
        // Ensure volume is mounted
        ensure_iso_volume(&sess, &host.name).await.map_err(srv_err)?;

        let (_, stdout, _) = crate::ssh::run_remote_script(&sess, "ls /mnt/isos/ | grep .iso || true", None).map_err(srv_err)?;
        let isos = stdout.lines().map(|s| s.to_string()).collect();
        
        Ok(isos)
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}
