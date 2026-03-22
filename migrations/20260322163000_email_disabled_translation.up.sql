INSERT INTO i18n_translations (key, locale, value) VALUES
('auth.register.email_disabled', 'en', 'Email registration is currently disabled. Please use direct registration.'),
('auth.register.email_disabled', 'ru', 'Регистрация через email временно отключена. Пожалуйста, используйте прямую регистрацию.')
ON CONFLICT (key, locale) DO UPDATE SET value = EXCLUDED.value;
