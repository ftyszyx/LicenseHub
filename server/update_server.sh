# stop server
docker compose -f docker-compose.yml down server
# pull new server image
docker compose -f docker-compose.yml pull server 
# start server
docker compose -f docker-compose.yml up -d --force-recreate server