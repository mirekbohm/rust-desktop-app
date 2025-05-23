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

pub struct DesktopApp {
    current_page: AppPage,
    data_store: DataStore,
    excel_exporter: ExcelExporter,
    updater: AppUpdater,
    update_status: String,
    update_receiver: Option<mpsc::Receiver<String>>,
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
                    ui.label(&self.update_status);
                });
            });
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
                    // Add some action
                    self.update_status = "Action 2 performed!".to_string();
                }
                if ui.button("💫 Action Button 3").clicked() {
                    // Add some action
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

                ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
                ui.label("Built with Rust and egui");
                ui.add_space(20.0);

                ui.label("Features:");
                ui.label("• Cross-platform desktop GUI");
                ui.label("• Excel export functionality");
                ui.label("• Automatic updates from GitHub");
                ui.label("• Modern, responsive interface");

                ui.add_space(30.0);
                ui.hyperlink_to("🌐 Visit GitHub Repository", "https://github.com/yourusername/your-repo");
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
        self.update_status = "Checking for updates...".to_string();
        
        let (tx, rx) = mpsc::channel();
        self.update_receiver = Some(rx);
        
        let ctx_clone = ctx.clone();
        
        // Spawn a thread to handle the async update check
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let updater = AppUpdater::new();
                match updater.check_for_updates().await {
                    Ok(has_update) => {
                        if has_update {
                            let _ = tx.send("Update available! Please check the repository.".to_string());
                        } else {
                            let _ = tx.send("No updates available.".to_string());
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(format!("Update check failed: {}", e));
                    }
                }
                ctx_clone.request_repaint();
            });
        });
    }

    fn check_update_result(&mut self) {
        if let Some(ref receiver) = self.update_receiver {
            if let Ok(message) = receiver.try_recv() {
                self.update_status = message;
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
    }
}
