use crate::{compare_hashes, world::TestWorld};
use cucumber_rust::{t, Steps};
use rust_test_client::stubs::cartesi_machine_manager::SessionReadMemoryResponse;
use std::boxed::Box;

pub fn steps() -> Steps<TestWorld> {
    let mut steps: Steps<TestWorld> = Steps::new();

    steps.given_regex_async(
        r#"the write request executed for cycle (\d+), starting address (\d+) and data (.+)"#,
        t!(|mut world, ctx| {
            let request = world.client_proxy.build_new_session_write_memory_request(
                ctx.matches[1].parse::<u64>().unwrap(),
                ctx.matches[2].parse::<u64>().unwrap(),
                ctx.matches[3].as_bytes().to_vec(),
            );
            let cl = world.client_proxy.grpc_client.as_mut().unwrap();
            if let Err(e) = cl.session_write_memory(request).await {
                world.response.insert(String::from("error"), Box::new(e));
            };
            world
        }),
    );

    steps.when_regex_async(
        r#"client asks server to read memory on cycle (\d+), starting on address (\d+) for length (\d+)"#,
    t!(|mut world, ctx| {
        let request = world.client_proxy.build_new_session_read_memory_request(
            ctx.matches[1].parse::<u64>().unwrap(),
            ctx.matches[2].parse::<u64>().unwrap(),
            ctx.matches[3].parse::<u64>().unwrap());
        match world.client_proxy.grpc_client.as_mut().unwrap().session_read_memory(request).await {
            Ok(val) => world.response.insert(String::from("response"), Box::new(val.into_inner())),
            Err(e) => world.response.insert(String::from("error"), Box::new(e)),
        };
        world
    }));

    steps.when_regex_async(
        r#"client asks server to write data (.+) on cycle (\d+), starting on address (\d+)"#,
        t!(|mut world, ctx| {
            let request = world.client_proxy.build_new_session_write_memory_request(
                ctx.matches[2].parse::<u64>().unwrap(),
                ctx.matches[3].parse::<u64>().unwrap(),
                ctx.matches[1].as_bytes().to_vec(),
            );
            if let Err(e) = world
                .client_proxy
                .grpc_client
                .as_mut()
                .unwrap()
                .session_write_memory(request)
                .await
            {
                world.response.insert(String::from("error"), Box::new(e));
            };
            world
        }),
    );

    steps.then_regex_async(
        r#"server returns read bytes ((\d|[A-Z])+)"#,
        t!(|mut world, ctx| {
            let response = world
                .response
                .get(&String::from("response"))
                .and_then(|x| x.downcast_ref::<SessionReadMemoryResponse>())
                .take()
                .unwrap();
            let read_bytes = &response.read_content.as_ref().unwrap().data;
            assert!(compare_hashes(read_bytes, &ctx.matches[1][..]));
            world
        }),
    );

    steps
}
