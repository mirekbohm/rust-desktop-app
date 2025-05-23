use crate::data::TableData;
use anyhow::Result;
use rust_xlsxwriter::*;
use std::path::PathBuf;

pub struct ExcelExporter;

impl ExcelExporter {
    pub fn new() -> Self {
        Self
    }

    pub fn export_data(&self, data: &[TableData]) -> Result<String> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // Set column headers
        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0xD3D3D3));

        worksheet.write_with_format(0, 0, "ID", &header_format)?;
        worksheet.write_with_format(0, 1, "Name", &header_format)?;
        worksheet.write_with_format(0, 2, "Value", &header_format)?;
        worksheet.write_with_format(0, 3, "Date", &header_format)?;

        // Write data
        for (row, item) in data.iter().enumerate() {
            let row = (row + 1) as u32;
            worksheet.write(row, 0, item.id)?;
            worksheet.write(row, 1, &item.name)?;
            worksheet.write(row, 2, item.value)?;
            worksheet.write(row, 3, item.date.format("%Y-%m-%d %H:%M:%S").to_string())?;
        }

        // Auto-fit columns
        worksheet.autofit();

        // Save to Downloads folder
        let mut path = dirs::download_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(format!("export_{}.xlsx", chrono::Local::now().format("%Y%m%d_%H%M%S")));

        workbook.save(&path)?;
        Ok(path.to_string_lossy().to_string())
    }
}
