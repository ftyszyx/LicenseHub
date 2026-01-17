# stop server
docker compose -f docker-compose.release.yml down server
# pull new server image
docker compose -f docker-compose.release.yml pull server 
# start server
docker compose -f docker-compose.release.yml up -d --force-recreate server