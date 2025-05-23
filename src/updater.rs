use anyhow::Result;
use self_update::cargo_crate_version;

pub struct AppUpdater {
    current_version: String,
}

impl AppUpdater {
    pub fn new() -> Self {
        Self {
            current_version: cargo_crate_version!().to_string(),
        }
    }

    pub async fn check_for_updates(&self) -> Result<Option<String>> {
        // Replace with your actual GitHub username and repository name
        let releases = self_update::backends::github::ReleaseList::configure()
            .repo_owner("mirekbohm") 
            .repo_name("rust-desktop-app") 
            .build()?
            .fetch()?;

        if let Some(latest_release) = releases.first() {
            let latest_version = &latest_release.version;
            
            // Simple version comparison
            if latest_version != &self.current_version {
                println!("New version available: {}", latest_version);
                return Ok(Some(latest_version.clone()));
            }
        }
        
        Ok(None)
    }

    pub async fn update_app(&self) -> Result<()> {
        let status = self_update::backends::github::Update::configure()
            .repo_owner("mirekbohm") 
            .repo_name("rust-desktop-app")
            .bin_name("desktop-app") 
            .show_download_progress(true)
            .current_version(cargo_crate_version!())
            .build()?
            .update()?;

        println!("Update status: `{}`!", status.version());
        Ok(())
    }
}
