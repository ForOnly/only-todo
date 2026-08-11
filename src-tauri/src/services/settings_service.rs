use crate::domain::{SettingsDto, UpdateSettingsDto};
use crate::errors::AppError;
use crate::infrastructure::database::Database;
use crate::repository::settings_repository::SettingsRepository;

pub struct SettingsService;

impl SettingsService {
    pub fn get(db: &Database) -> Result<SettingsDto, AppError> {
        SettingsRepository::get(db)
    }

    pub fn update(db: &Database, dto: UpdateSettingsDto) -> Result<SettingsDto, AppError> {
        SettingsRepository::update(db, &dto)
    }

    pub fn is_notification_enabled(db: &Database) -> Result<bool, AppError> {
        Ok(SettingsRepository::get(db)?.notification_enabled)
    }

    pub fn float_always_on_top(db: &Database) -> Result<bool, AppError> {
        Ok(SettingsRepository::get(db)?.float_always_on_top)
    }
}
