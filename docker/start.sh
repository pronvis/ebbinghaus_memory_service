#!/bin/sh
docker run --rm -e POSTGRES_PASSWORD=simple -e POSTGRES_USER=postgres -e POSTGRES_DB=diesel_demo -p 5432:5432 -d --name postgres postgres
