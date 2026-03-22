# Используем официальный образ Node.js
FROM node:20-alpine AS build

# Устанавливаем рабочую директорию
WORKDIR /app

# Копируем package.json и package-lock.json (или yarn.lock)
COPY package*.json ./

# Устанавливаем зависимости
RUN pnpm install

# Копируем остальные файлы проекта
COPY . .

# Собираем проект
RUN pnpm build-only

# Используем легкий образ nginx для раздачи статики
FROM nginx:alpine

# Копируем собранные файлы из предыдущего этапа
COPY --from=build /app/dist /usr/share/nginx/html

# Копируем конфигурационный файл nginx
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Открываем 80 порт
EXPOSE 80

# Запускаем nginx
CMD ["nginx", "-g", "daemon off;"]