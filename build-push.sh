PORT=22
USER=root
SERVER=origin.kite.sunnysab.cn  # Changed by yourself.

# build
cargo build --release

# copy to server
scp -P $PORT target/release/kite-server $USER@$SERVER:/var/kite/kite-server-v2.tmp

# update
ssh -p $PORT $USER@$SERVER \
    "systemctl stop kite2; \
    mv /var/kite/kite-server-v2 /var/kite/kite-server-v2.bak; \
    mv /var/kite/kite-server-v2.tmp /var/kite/kite-server-v2; \
    systemctl start kite2; \
    sleep 1; \
    systemctl status kite2"

if [[ $? != "0" ]] ;then
    echo "更新失败, 正在回退."

    ssh -p $PORT $USER@$SERVER \
        "mv /var/kite/kite-server-v2.bak /var/kite/kite-server-v2; \
        systemctl start kite2"
fi


echo "Exit."
