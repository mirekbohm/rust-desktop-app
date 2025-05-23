use eframe::egui;
use std::sync::mpsc;
use std::thread;

use crate::data::{DataStore, TableData};
use crate::export::ExcelExporter;
use crate::updater::AppUpdater;

#[derive(Default)]
pub enum AppPage {
    #[default]
    Home,
    DataTable,
    Settings,
    About,
}

#[derive(Default)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    Available(String), // Contains the new version number
    Downloading,
    Downloaded,
    Error(String),
}

pub struct DesktopApp {
    current_page: AppPage,
    data_store: DataStore,
    excel_exporter: ExcelExporter,
    updater: AppUpdater,
    update_status: String,
    update_receiver: Option<mpsc::Receiver<UpdateResult>>,
    
    // Update dialog state
    update_state: UpdateState,
    show_update_dialog: bool,
    available_version: String,
}

#[derive(Debug)]
enum UpdateResult {
    UpdateAvailable(String),
    NoUpdate,
    UpdateDownloaded,
    Error(String),
}

impl DesktopApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_page: AppPage::default(),
            data_store: DataStore::new(),
            excel_exporter: ExcelExporter::new(),
            updater: AppUpdater::new(),
            update_status: "Ready".to_string(),
            update_receiver: None,
            update_state: UpdateState::default(),
            show_update_dialog: false,
            available_version: String::new(),
        }
    }

    fn show_menubar(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menubar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("📁 Open").clicked() {
                        // Implement file opening logic
                        ui.close_menu();
                    }
                    if ui.button("💾 Save").clicked() {
                        // Implement save logic
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("📤 Export to Excel").clicked() {
                        self.export_to_excel();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("❌ Exit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("🏠 Home").clicked() {
                        self.current_page = AppPage::Home;
                        ui.close_menu();
                    }
                    if ui.button("📊 Data Table").clicked() {
                        self.current_page = AppPage::DataTable;
                        ui.close_menu();
                    }
                    if ui.button("⚙️ Settings").clicked() {
                        self.current_page = AppPage::Settings;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("🔄 Check for Updates").clicked() {
                        self.check_for_updates(ctx);
                        ui.close_menu();
                    }
                    if ui.button("ℹ️ About").clicked() {
                        self.current_page = AppPage::About;
                        ui.close_menu();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    match &self.update_state {
                        UpdateState::Checking => ui.label("🔄 Checking for updates..."),
                        UpdateState::Downloading => ui.label("⬇️ Downloading update..."),
                        UpdateState::Downloaded => ui.label("✅ Update ready! Restart app."),
                        UpdateState::Error(_) => ui.label("❌ Update check failed"),
                        _ => ui.label(&self.update_status),
                    };
                });
            });
        });
    }

    fn show_update_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_update_dialog {
            return;
        }

        egui::Window::new("Update Available")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(400.0);
                
                // Header with icon and title
                ui.horizontal(|ui| {
                    ui.label("🎉");
                    ui.heading("New Version Available!");
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // Version information
                ui.horizontal(|ui| {
                    ui.label("Current version:");
                    ui.strong(env!("CARGO_PKG_VERSION"));
                });
                
                ui.horizontal(|ui| {
                    ui.label("New version:");
                    ui.strong(&self.available_version);
                });

                ui.add_space(15.0);

                // Update description
                ui.label("A new version of the application is available for download.");
                ui.label("Would you like to update now?");

                ui.add_space(20.0);

                // Show different content based on update state
                match &self.update_state {
                    UpdateState::Available(_) => {
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("❌ Later").clicked() {
                                    self.show_update_dialog = false;
                                    self.update_state = UpdateState::Idle;
                                }
                                
                                if ui.button("⬇️ Update Now").clicked() {
                                    self.download_update(ctx);
                                }
                            });
                        });
                    }
                    
                    UpdateState::Downloading => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Downloading update...");
                        });
                        ui.add_space(10.0);
                        ui.label("Please wait while the update is being downloaded.");
                    }
                    
                    UpdateState::Downloaded => {
                        ui.colored_label(egui::Color32::GREEN, "✅ Update downloaded successfully!");
                        ui.add_space(10.0);
                        ui.label("Please restart the application to complete the update.");
                        
                        ui.add_space(15.0);
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("🔄 Restart Now").clicked() {
                                    // Restart the application
                                    std::process::exit(0);
                                }
                                
                                if ui.button("📋 Continue").clicked() {
                                    self.show_update_dialog = false;
                                    self.update_state = UpdateState::Idle;
                                }
                            });
                        });
                    }
                    
                    UpdateState::Error(error) => {
                        ui.colored_label(egui::Color32::RED, "❌ Update failed!");
                        ui.add_space(5.0);
                        ui.label(format!("Error: {}", error));
                        
                        ui.add_space(15.0);
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("❌ Close").clicked() {
                                    self.show_update_dialog = false;
                                    self.update_state = UpdateState::Idle;
                                }
                                
                                if ui.button("🔄 Retry").clicked() {
                                    self.download_update(ctx);
                                }
                            });
                        });
                    }
                    
                    _ => {}
                }
            });
    }

    fn show_home_page(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to Desktop Application");
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if ui.button("🎯 Action Button 1").clicked() {
                    self.data_store.add_sample_data();
                }
                if ui.button("🚀 Action Button 2").clicked() {
                    self.update_status = "Action 2 performed!".to_string();
                }
                if ui.button("💫 Action Button 3").clicked() {
                    self.update_status = "Action 3 performed!".to_string();
                }
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);

            ui.heading("Quick Stats");
            egui::Grid::new("stats_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Total Records:");
                    ui.label(format!("{}", self.data_store.get_record_count()));
                    ui.end_row();

                    ui.label("Last Updated:");
                    ui.label(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                    ui.end_row();

                    ui.label("Version:");
                    ui.label(env!("CARGO_PKG_VERSION"));
                    ui.end_row();
                });
        });
    }

    fn show_data_table_page(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Data Table");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("➕ Add Sample Data").clicked() {
                    self.data_store.add_sample_data();
                }
                if ui.button("🗑️ Clear Data").clicked() {
                    self.data_store.clear_data();
                }
                if ui.button("📤 Export to Excel").clicked() {
                    self.export_to_excel();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Show table
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("data_table")
                    .num_columns(4)
                    .spacing([10.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // Header
                        ui.strong("ID");
                        ui.strong("Name");
                        ui.strong("Value");
                        ui.strong("Date");
                        ui.end_row();

                        // Data rows
                        for item in self.data_store.get_all_data() {
                            ui.label(item.id.to_string());
                            ui.label(&item.name);
                            ui.label(format!("{:.2}", item.value));
                            ui.label(item.date.format("%Y-%m-%d").to_string());
                            ui.end_row();
                        }
                    });
            });
        });
    }

    fn show_settings_page(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Settings");
            ui.add_space(20.0);

            egui::Grid::new("settings_grid")
                .num_columns(2)
                .spacing([40.0, 10.0])
                .show(ui, |ui| {
                    ui.label("Auto-update:");
                    ui.checkbox(&mut true, "Enable automatic updates");
                    ui.end_row();

                    ui.label("Theme:");
                    egui::ComboBox::from_label("")
                        .selected_text("Dark")
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut "Dark", "Dark", "Dark");
                            ui.selectable_value(&mut "Light", "Light", "Light");
                        });
                    ui.end_row();

                    ui.label("Language:");
                    egui::ComboBox::from_label("")
                        .selected_text("English")
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut "English", "English", "English");
                            ui.selectable_value(&mut "Spanish", "Spanish", "Spanish");
                        });
                    ui.end_row();
                });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);

            ui.heading("Update Settings");
            ui.add_space(10.0);

            if ui.button("🔄 Check for Updates Now").clicked() {
                self.check_for_updates(ctx);
            }

            ui.add_space(10.0);
            ui.label(format!("Update Status: {}", self.update_status));
        });
    }

    fn show_about_page(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("About Desktop Application");
                ui.add_space(20.0);

                ui.label(format!("Version: {} (Updated!)", env!("CARGO_PKG_VERSION")));
                ui.label("Built with Rust, egui and LOVE");
                ui.label("🎉 This is the updated version!");
                ui.add_space(20.0);

                ui.label("Features:");
                ui.label("• Windows compatible desktop GUI");
                ui.label("• Excel export functionality");
                ui.label("• Automatic updates from GitHub");
                ui.label("• Modern, responsive interface");

                ui.add_space(30.0);
                ui.hyperlink_to("🌐 Visit GitHub Repository", "https://github.com/mirekbohm/rust-desktop-app");
            });
        });
    }

    fn export_to_excel(&mut self) {
        let data = self.data_store.get_all_data();
        match self.excel_exporter.export_data(&data) {
            Ok(path) => {
                self.update_status = format!("Exported to: {}", path);
            }
            Err(e) => {
                self.update_status = format!("Export failed: {}", e);
            }
        }
    }

    fn check_for_updates(&mut self, ctx: &egui::Context) {
        self.update_state = UpdateState::Checking;
        
        let (tx, rx) = mpsc::channel();
        self.update_receiver = Some(rx);
        
        let ctx_clone = ctx.clone();
        
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let updater = AppUpdater::new();
                match updater.check_for_updates().await {
                    Ok(Some(version)) => {
                        let _ = tx.send(UpdateResult::UpdateAvailable(version));
                    }
                    Ok(None) => {
                        let _ = tx.send(UpdateResult::NoUpdate);
                    }
                    Err(e) => {
                        let _ = tx.send(UpdateResult::Error(e.to_string()));
                    }
                }
                ctx_clone.request_repaint();
            });
        });
    }

    fn download_update(&mut self, ctx: &egui::Context) {
        self.update_state = UpdateState::Downloading;
        
        let (tx, rx) = mpsc::channel();
        self.update_receiver = Some(rx);
        
        let ctx_clone = ctx.clone();
        
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let updater = AppUpdater::new();
                match updater.update_app().await {
                    Ok(_) => {
                        let _ = tx.send(UpdateResult::UpdateDownloaded);
                    }
                    Err(e) => {
                        let _ = tx.send(UpdateResult::Error(e.to_string()));
                    }
                }
                ctx_clone.request_repaint();
            });
        });
    }

    fn check_update_result(&mut self) {
        if let Some(ref receiver) = self.update_receiver {
            if let Ok(result) = receiver.try_recv() {
                match result {
                    UpdateResult::UpdateAvailable(version) => {
                        self.available_version = version.clone();
                        self.update_state = UpdateState::Available(version);
                        self.show_update_dialog = true;
                        self.update_status = "Update available!".to_string();
                    }
                    UpdateResult::NoUpdate => {
                        self.update_state = UpdateState::Idle;
                        self.update_status = "No updates available.".to_string();
                    }
                    UpdateResult::UpdateDownloaded => {
                        self.update_state = UpdateState::Downloaded;
                        self.update_status = "Update downloaded!".to_string();
                    }
                    UpdateResult::Error(error) => {
                        self.update_state = UpdateState::Error(error.clone());
                        self.update_status = format!("Update failed: {}", error);
                    }
                }
                self.update_receiver = None;
            }
        }
    }
}

impl eframe::App for DesktopApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Check for update results
        self.check_update_result();
        
        self.show_menubar(ctx, frame);

        match self.current_page {
            AppPage::Home => self.show_home_page(ctx),
            AppPage::DataTable => self.show_data_table_page(ctx),
            AppPage::Settings => self.show_settings_page(ctx),
            AppPage::About => self.show_about_page(ctx),
        }

        // Show update dialog if needed
        self.show_update_dialog(ctx);
    }
}
