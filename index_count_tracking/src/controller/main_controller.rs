use crate::common::*;


use crate::traits::{
    service_traits::{
        notification_service::*, query_service::*
    }
};

#[derive(Debug, new)]
pub struct MainController<N: NotificationService, TQ: QueryService, MQ: QueryService> {
    notification_service: N,
    target_query_service: TQ,
    mon_query_service: MQ
}

impl<N: NotificationService, TQ: QueryService, MQ: QueryService> MainController<N, TQ, MQ> {
    
    #[doc = ""]
    pub async fn main_task(&self) -> anyhow::Result<()> {
        
        let mut ticker: Interval = interval(Duration::from_secs(10));

        loop {
            
            ticker.tick().await; 
            
            /* 1. 특정 인덱스 문서 개수를 카운트 -> 10초에 한번씩? */
            

            /* 2. 집계를 하여 문제가 있는 경우에는 알람을 보냄 */
        }


        Ok(())
    }
}