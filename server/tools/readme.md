#install
pip install -r requirements.txt -i https://pypi.tuna.tsinghua.edu.cn/simple


#use
python migrate.py --source-dsn "postgresql://username:password@host:port/database" --target-dsn "postgresql://username:password@host:port/database"

python migrate.py --source-dsn "postgresql://apphub:dasfjaksdigj@8.134.157.107:5432/apphub" --target-dsn "postgresql://apphub:4P8x5QMyX38nyrGy@8.134.157.107:20115/apphub"
