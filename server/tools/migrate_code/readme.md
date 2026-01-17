#install
pip install -r requirements.txt -i https://pypi.tuna.tsinghua.edu.cn/simple


#use
python migrate.py --source-dsn "postgresql://username:password@host:port/database" --target-dsn "postgresql://username:password@host:port/database"

