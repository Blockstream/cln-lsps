use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

type MatcherState<RequestId, Response> = HashMap<RequestId, Arc<Mutex<RequestState<Response>>>>;

/// Matches requests and responses based on `RequestId` coming from
/// multiple sources.
///
/// The caller should ensure that a `RequestId` is never re-used.
///
/// When a message is sent the user can call `process_request` which returns a
/// `Future<Output=Response>`. Calling `process_response` using the same `RequestId`
/// will awake the future and set the state to ready.
///
/// The implementation is thread-safe due to the over-use of Arc<Mutex<_>>. Performance,
/// on large number of requests and responses might be suboptimal.
#[derive(Clone)]
pub struct RequestResponseMatcher<RequestId, Response> {
    futures: Arc<Mutex<MatcherState<RequestId, Response>>>,
}

impl<RequestId, Response> RequestResponseMatcher<RequestId, Response> {
    /// Initializes a new RequestResponseMatcher
    pub fn new() -> Self {
        let futures = HashMap::new();
        return Self {
            futures: Arc::new(Mutex::new(futures)),
        };
    }

    #[cfg(test)]
    fn consume(self) -> Arc<Mutex<MatcherState<RequestId, Response>>> {
        self.futures
    }
}

impl<RequestId, Response> RequestResponseMatcher<RequestId, Response>
where
    RequestId: Hash + Eq + Clone,
    Response: Clone,
{
    /// Processes an incoming request and returns a Future.
    ///
    /// The future will be `Ready` once process_response has been
    /// called.
    pub fn process_request(&mut self, id: RequestId) -> impl Future<Output = Response> {
        let future = RequestFuture::new(id.clone(), self.futures.clone());
        let _ = self
            .futures
            .lock()
            .unwrap()
            .insert(id, future.state.clone());
        future
    }

    /// Processes an incoming Response and returns `true` if a matching request
    /// exists.
    pub fn process_response(&mut self, request_id: &RequestId, response: Response) -> bool {
        let mut mutex = self.futures.lock().unwrap().remove(&request_id);

        match &mut mutex {
            Some(m) => {
                let mut state = m.lock().unwrap();
                state.response = Some(response);
                state.waker.as_mut().map(|w| w.clone().wake());
                true
            }
            None => false,
        }
    }
}

/// The `RequestState` is shared between the `RequestResponseMatcher` and the
/// `Future` corresponding to that request.
///
/// It contains a `waker` that can be used to wake the `Future` and the `data`
/// that corresponds to the response
struct RequestState<Resp> {
    pub waker: Option<Waker>,
    pub response: Option<Resp>,
}

impl<Resp> RequestState<Resp> {
    fn new() -> Self {
        Self {
            waker: None,
            response: None,
        }
    }
}

struct RequestFuture<RequestId, Resp>
where
    RequestId: Eq + Hash,
{
    /// Performance improvement:
    /// We are running a single thread and don't need a Mutex.
    /// UnsafeCell should work for our plugin and client implementation.
    /// However, users of this library might abuse it
    state: Arc<Mutex<RequestState<Resp>>>,

    request_id: RequestId,
    context: Arc<Mutex<HashMap<RequestId, Arc<Mutex<RequestState<Resp>>>>>>,
}

impl<RequestId, Response> Drop for RequestFuture<RequestId, Response>
where
    RequestId: Eq + Hash,
{
    fn drop(&mut self) {
        let mut mapper = self.context.lock().unwrap();
        mapper.remove(&self.request_id);
    }
}

impl<RequestId, Response> Future for RequestFuture<RequestId, Response>
where
    Response: Clone,
    RequestId: Eq + Hash,
{
    type Output = Response;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<<Self as Future>::Output> {
        let mut shared_state = self.state.lock().unwrap();

        match shared_state.response.as_mut() {
            Some(d) => Poll::Ready(d.clone()),
            None => {
                shared_state.waker = Some(context.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl<RequestId, Response> RequestFuture<RequestId, Response>
where
    RequestId: Eq + Hash,
{
    fn new(request_id: RequestId, context: Arc<Mutex<MatcherState<RequestId, Response>>>) -> Self {
        RequestFuture {
            state: Arc::new(Mutex::new(RequestState::new())),
            context,
            request_id,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::future::Future;

    fn receive<RequestId, Response>(future: &mut RequestFuture<RequestId, Response>, data: Response)
    where
        RequestId: Hash + Eq,
    {
        let mut shared_state = future.state.lock().unwrap();
        shared_state.response = Some(data);

        match &shared_state.waker {
            Some(w) => w.clone().wake(),
            None => (),
        }
    }

    #[test]
    fn test_custom_message_future_poll() {
        let matcher_state = Arc::new(Mutex::new(HashMap::new()));
        let mut future = RequestFuture::<String, _>::new("msg_id_001".into(), matcher_state);

        // Initialize a fake context
        let waker = futures::task::noop_waker_ref();
        let mut cx = std::task::Context::from_waker(waker);

        // Verify that the future is pending
        {
            if let Poll::Ready(_) = (Pin::new(&mut future)).poll(&mut cx) {
                panic!("RequestFuture should be pending because no message was received")
            }
        }

        // Send a message to to future and assert the future is ready
        {
            let payload = "message payload";
            receive(&mut future, payload);

            let result = (Pin::new(&mut future)).poll(&mut cx);
            match result {
                Poll::Pending => panic!("Future remains pending but message was received"),
                Poll::Ready(p) => {
                    assert_eq!(payload, p);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_await_response_feature() {
        let matcher_state = Arc::new(Mutex::new(HashMap::new()));
        let payload = "message content";
        let mut future = RequestFuture::<String, _>::new("msg_id_001".into(), matcher_state);

        // Send the message we are waiting for
        receive(&mut future, payload);

        let result = future.await;

        assert_eq!(result, payload, "The payload in the message doesn't match");
    }

    #[tokio::test]
    async fn test_match_request_response() {
        let mut matcher = RequestResponseMatcher::<String, u64>::new();

        // Create some requests and create the corresponding future
        let req_1 = matcher.process_request(String::from("request_1"));
        let req_2 = matcher.process_request(String::from("request_2"));
        let req_3 = matcher.process_request(String::from("request_3"));

        // We process the response for message 3
        // We intentionally handle them out of order in this test\
        //
        // This helps us to test that the correct message is matched
        // to the expected response
        matcher.process_response(&String::from("request_3"), 3);

        // A future depends on a waker. The process_response
        // message should call waker.wake() so that the run-time
        // knows it can call `.poll` again when it's ready.
        //
        // To ensure we test this behavior we have to ensure
        // we
        // - first call `await`
        // - call `process_response` later
        //
        // We'll use the ready signal later. Once the ready_signal
        // is set to `true` we can start calling `process_response`
        let ready_signal = Arc::new(Mutex::new(false));
        let s = ready_signal.clone();
        tokio::spawn(async move {
            // Wait for ready signal
            loop {
                if *s.lock().unwrap() {
                    break;
                }
            }

            // We'll send a mix of poorly ordered messages and
            // "ghost" responses that don't match any request
            matcher.process_response(&String::from("ghost_a"), 1);
            matcher.process_response(&String::from("ghost_b"), 1);
            matcher.process_response(&String::from("request_1"), 1);
            matcher.process_response(&String::from("request_2"), 2);
        });

        // This function sets a `ready_signal` to `true`
        async fn set_ready_func(ready_signal: Arc<Mutex<bool>>) {
            *ready_signal.lock().unwrap() = true
        }
        let set_ready_future = set_ready_func(ready_signal);

        // Await all messages and confirm the correct request was matched
        // to the correct resposne
        let (resp1, resp2, resp3, ()) = futures::join!(req_1, req_2, req_3, set_ready_future);

        assert_eq!(resp1, 1);
        assert_eq!(resp2, 2);
        assert_eq!(resp3, 3);
    }

    #[tokio::test]
    /// Test that a all state is dropped when a future is dropped
    ///
    /// This is especially useful when implementing time-outs.
    /// You can wrap the future inside a tokio::TimeOut future.
    /// If a time-out would occur all clean-up happens auto-magically.
    async fn test_dropped_futures_are_removed_from_hashmap() {
        let mut matcher = RequestResponseMatcher::<String, u64>::new();

        // Create some requests and create the corresponding future
        let _req_1 = matcher.process_request(String::from("request_1"));

        // Create a new scope which will drop req_2
        {
            let _req_2 = matcher.process_request(String::from("request_2"));
        }

        let _req_3 = matcher.process_request(String::from("request_3"));

        let mutex_map = matcher.consume();
        let map = mutex_map.lock().unwrap();
        assert!(map.contains_key("request_1")); // Still in the HashMap
        assert!(!map.contains_key("request_2")); // Has been dropped
        assert!(map.contains_key("request_3")); // Still in the HashMap
    }
}
