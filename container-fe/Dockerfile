FROM node:23 AS build

WORKDIR /app

RUN npm install -g @angular/cli

COPY package*.json ./

RUN npm install


COPY . .

RUN ls ; ng build --configuration=production

FROM nginx:latest

COPY --from=build /app/dist/shell/browser /usr/share/nginx/html
COPY nginx/templates/* /etc/nginx/templates/

EXPOSE 8080
