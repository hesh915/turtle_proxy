use slog::{error, info, Logger};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::sync::mpsc;
use tokio::time::{delay_for, timeout, Duration};

#[derive(Debug)]
enum MessageSession {
    NewSession,
    DelSession,
}

pub async fn start_local(
    logger_: &Logger,
    local_listen_addr: &str,
    tunnel_server_addr: &str,
) -> std::io::Result<()> {
    info!(
        logger_,
        "start_client local_listen_addr={},tunnel_server_addr={}",
        local_listen_addr,
        tunnel_server_addr
    );

    let (session_tx, mut session_rx) = mpsc::channel::<MessageSession>(100);

    //1. listen local port
    let logger = logger_.clone();
    let mut listener = match TcpListener::bind(local_listen_addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!(logger, "listen bind:{}", e);
            return Err(e);
        }
    };
    tokio::spawn(async move {
        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok((t, s)) => (t, s),
                Err(e) => {
                    error!(logger, "accept error:{}", e);
                    return;
                }
            };

            let mut tx = session_tx.clone();
            let logger = logger.clone();
            tokio::spawn(async move {
                //process_tcp(tx1, socket).await;
                info!(logger, "socket={:?}", socket);

                let tmp = MessageSession::NewSession;
                tx.send(tmp).await.unwrap();

                let (mut reader, mut writer) = socket.split();
                tokio::io::copy(&mut reader, &mut writer).await.unwrap();
                //socket.read_to_end(buf);
                //socket.read(buf)

                let tmp = MessageSession::DelSession;
                tx.send(tmp).await.unwrap();
            });
        }
    });

    //2. connect tunnel to remote
    let logger = logger_.clone();
    let tunnel_server_addr = tunnel_server_addr.to_owned();
    tokio::spawn(async move {
        loop {
            let tunnel_server_addr = tunnel_server_addr.to_owned();
            let mut socket = match timeout(
                Duration::from_secs(5),
                TcpStream::connect(tunnel_server_addr),
            )
            .await
            {
                Ok(Ok(socket)) => socket,
                Ok(Err(e)) => {
                    error!(logger, "error connect:{}", e);
                    delay_for(Duration::from_secs(5)).await;
                    continue;
                }
                Err(e) => {
                    error!(logger, "error timeout:{}", e);
                    continue;
                }
            };
            info!(
                logger,
                "Connect remote server successful, tunnel is ok, connection:[{:?}]", socket
            );
            //TODO Read

            let (mut reader, mut _writer) = socket.split();
            let mut buffer = [0; 128];

            loop {
                tokio::select! {
                    length = reader.read(&mut buffer[..])=>{
                        match length{
                            Ok(len)=>{
                                let str1 = String::from_utf8_lossy(&buffer);
                                info!(logger, "got data len = {:?}, msg = {:?}", len,str1);
                                if len == 0{
                                    //todo set state close
                                    break;
                                }
                                //todo read data

                            }
                            Err(e) => println!("{:?}", e),
                        }
                    },
                    msg = session_rx.recv()=>{
                        info!(logger, "got session msg = {:?}", msg);
                    },
                }
            }
        }
    });
    Ok(())
}
