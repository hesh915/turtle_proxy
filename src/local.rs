use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::{delay_for, timeout, Duration};
//use tokio::prelude::*;
use slog::{error, info, Logger};

#[derive(Debug)]
enum MessageSession {
    NewSession,
    DelSession,
}
#[derive(Debug)]
enum MessageTunnel {
    TunnelConnected,
    _TunnelDisconnected,
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
    let (mut tunnel_tx, mut tunnel_rx) = mpsc::channel::<MessageTunnel>(100);

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
            let (mut socket, _) = listener.accept().await.unwrap();
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
            let _socket = match timeout(
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


            let tmp = MessageTunnel::TunnelConnected;
            tunnel_tx.send(tmp).await.unwrap();


            //todo delete
            loop {
                delay_for(Duration::from_secs(1)).await;
                //println!("heart beat !");
            }
        }
    });

    //3. handle session msg
    let logger = logger_.clone();
    tokio::spawn(async move {
        loop {
            for msg in session_rx.recv().await {
                info!(logger, "got session msg = {:?}", msg);
            }
        }
    });

    //4. handle tunnel msg
    let logger = logger_.clone();
    tokio::spawn(async move {
        loop {
            for msg in tunnel_rx.recv().await {
                info!(logger, "got tunnel msg = {:?}", msg);
            }
        }
    });
    Ok(())
}
