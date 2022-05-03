use erl_dist::node::NodeName;
use erl_dist::term::{Atom, List, Term};
use futures::channel::oneshot;

// use erl_dist::term::Term;
// use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SystemVersion(String);

impl SystemVersion {
    pub fn get(&self) -> &str {
        &self.0
    }
}

pub fn find_cookie() -> anyhow::Result<String> {
    if let Some(dir) = dirs::home_dir().filter(|dir| dir.join(".erlang.cookie").exists()) {
        let cookie = std::fs::read_to_string(dir.join(".erlang.cookie"))?;
        Ok(cookie)
    } else {
        anyhow::bail!("Could not find the cookie file $HOME/.erlang.cookie. Please specify `-cookie` arg instead.");
    }
}

#[derive(Debug)]
pub struct RpcClient {
    handle: erl_rpc::RpcClientHandle,
    err_rx: oneshot::Receiver<erl_rpc::RunError>,
}

impl RpcClient {
    pub async fn connect(erlang_node: &NodeName, cookie: &str) -> anyhow::Result<Self> {
        let client = erl_rpc::RpcClient::connect(&erlang_node.to_string(), cookie).await?;
        let handle = client.handle();
        let (err_tx, err_rx) = oneshot::channel();
        smol::spawn(async {
            if let Err(e) = client.run().await {
                let _ = err_tx.send(e);
            }
        })
        .detach();

        Ok(Self { handle, err_rx })
    }

    pub async fn get_system_version(&mut self) -> anyhow::Result<SystemVersion> {
        let result = self
            .handle
            .call(
                "erlang".into(),
                "system_info".into(),
                List::from(vec![Atom::from("system_version").into()]),
            )
            .await?;
        term_to_string(result).map(SystemVersion)
    }
}
//     pub async fn get_msacc_stats(
//         &mut self,
//         duration: Duration,
//     ) -> anyhow::Result<self::msacc::MsaccData> {
//         if let Ok(Some(e)) = self.err_rx.try_recv() {
//             return Err(e.into());
//         }
//         self::msacc::get_msacc_stats(self.handle.clone(), duration).await
//     }

//     pub async fn get_memory_stats(&mut self) -> anyhow::Result<self::memory::MemoryStats> {
//         self::memory::get_memory_stats(self.handle.clone()).await
//     }

//     pub async fn get_stats(&mut self) -> anyhow::Result<self::stats::Stats> {
//         self::stats::Stats::collect(self.handle.clone()).await
//     }
// }

// impl Drop for RpcClient {
//     fn drop(&mut self) {
//         self.handle.terminate();
//     }
// }

// pub fn term_to_u64(term: Term) -> anyhow::Result<u64> {
//     let v = match term {
//         Term::FixInteger(v) => v.value.try_into()?,
//         Term::BigInteger(v) => v.value.try_into()?,
//         v => anyhow::bail!("{} is not an integer", v),
//     };
//     Ok(v)
// }

fn term_to_string(term: Term) -> anyhow::Result<String> {
    if let Term::List(list) = term {
        let bytes = list
            .elements
            .into_iter()
            .map(term_to_u8)
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(String::from_utf8(bytes)?)
    } else {
        anyhow::bail!("expected a string, but got {}", term);
    }
}

fn term_to_u8(term: Term) -> anyhow::Result<u8> {
    if let Term::FixInteger(v) = term {
        Ok(u8::try_from(v.value)?)
    } else {
        anyhow::bail!("expected an integer, but got {}", term)
    }
}
