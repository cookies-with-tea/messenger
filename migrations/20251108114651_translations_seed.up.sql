INSERT INTO i18n_translations (key, locale, value) VALUES
-- user.*
('user.created', 'en', 'User successfully created'),
('user.created', 'ru', 'Пользователь успешно создан'),

('user.deleted', 'en', 'User successfully deleted'),
('user.deleted', 'ru', 'Пользователь успешно удалён'),

('user.not_found', 'en', 'User not found'),
('user.not_found', 'ru', 'Пользователь не найден'),

('user.phone_exists', 'en', 'User with this phone number already exists'),
('user.phone_exists', 'ru', 'Пользователь с таким номером телефона уже существует'),
('user.phone_invalid', 'en', 'Phone number is invalid'),
('user.phone_invalid', 'ru', 'Номер телефона неверный'),

('user.password_hash_error', 'en', 'Password hashing error'),
('user.password_hash_error', 'ru', 'Ошибка хеширования пароля'),

('user.check_exists_error', 'en', 'Failed to check user existence'),
('user.check_exists_error', 'ru', 'Не удалось проверить существование пользователя'),

('user.email_exists', 'en', 'User with this email already exists'),
('user.email_exists', 'ru', 'Пользователь с таким email уже существует'),
('user.email_invalid', 'en', 'Email is invalid'),
('user.email_invalid', 'ru', 'Email неверный'),

-- auth.*
('auth.register.check_email', 'en', 'Registration initiated. Please check your email to confirm.'),
('auth.register.check_email', 'ru', 'Регистрация начата. Пожалуйста, проверьте вашу почту для подтверждения.'),

('auth.register.code_not_found', 'en', 'Registration code not found'),
('auth.register.code_not_found', 'ru', 'Код подтверждения не найден'),

('auth.register.code_expired', 'en', 'Registration code has expired'),
('auth.register.code_expired', 'ru', 'Срок действия кода подтверждения истёк'),

('auth.register.code_invalid', 'en', 'Invalid registration code'),
('auth.register.code_invalid', 'ru', 'Неверный код подтверждения'),

('auth.register.code_valid', 'en', 'Registration code is valid'),
('auth.register.code_valid', 'ru', 'Код подтверждения действителен'),

('auth.invalid_credentials', 'en', 'Invalid login or password'),
('auth.invalid_credentials', 'ru', 'Неверный логин или пароль'),

('auth.database_error', 'en', 'Database error'),
('auth.database_error', 'ru', 'Ошибка запроса к базе данных'),

('auth.refresh_invalid_or_expired', 'en', 'Refresh token is invalid or expired'),
('auth.refresh_invalid_or_expired', 'ru', 'Refresh-токен недействителен или истёк'),

('auth.refresh_db_error', 'en', 'Database error while checking refresh token'),
('auth.refresh_db_error', 'ru', 'Ошибка при проверке refresh-токена'),

('auth.logout_db_error', 'en', 'Database error while logging out'),
('auth.logout_db_error', 'ru', 'Ошибка при удалении refresh-токена'),

('auth.login_success', 'en', 'Login successful'),
('auth.login_success', 'ru', 'Авторизация успешна'),

('auth.unauthorized', 'en', 'Unauthorized'),
('auth.unauthorized', 'ru', 'Не авторизован'),

('auth.refresh_success', 'en', 'Token successfully refreshed'),
('auth.refresh_success', 'ru', 'Токен успешно обновлён'),

('auth.logout_success', 'en', 'Logout successful'),
('auth.logout_success', 'ru', 'Выход выполнен успешно'),

('auth.emal_sent', 'en', 'An email has been sent to your inbox. Please {span}click the link{/span} in the email to confirm your email address.'),
('auth.emal_sent', 'ru', 'Вам на почту отправлено письмо. Пожалуйста, {span}перейдите по ссылке{/span} в письме, чтобы подтвердить ваш E-mail.'),

-- general.*
('general.avg_sum', 'en', 'Average check: {sum}'),
('general.avg_sum', 'ru', 'Средний чек: {sum}'),

('general.avg_sum_2', 'en', 'Average check: {br}{sum}'),
('general.avg_sum_2', 'ru', 'Средний чек: {br}{sum}'),

('general.internal_error', 'en', 'Internal server error'),
('general.internal_error', 'ru', 'Внутренняя ошибка сервера'),

('general.email_failed', 'en', 'Email sending failed'),
('general.email_failed', 'ru', 'Не удалось отправить письмо'),

('general.db_error', 'en', 'Internal error'),
('general.db_error', 'ru', 'Внутренняя ошибка')

ON CONFLICT (key, locale) DO NOTHING;
