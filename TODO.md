# TODO

## Docs & Finishing Touches

- Create high-quality user docs instead of the AI slop that it is now
- Create a high-quality default config file with useful default expressions
- Add more tests

## Features

- Provide a mechanism to clear cached data
- Implement retry logic when downloading stuff from the interwebs in general using seatbelt
- Move the "opening vulnerability database" op to happen at the same time as invoking the crates provider to amortize the cost of opening the database
- Add a way to skip certain providers via CLI flags
- Add general-purpose timeout handling for providers that take too long to respond

## Performance Cleanup

- Look for opportunities to use iterators instead of slices or vectors to reduce the allocation rate
- Look for opportunities to avoid cloning to reduce the allocation rate and/or use of atomic ops
- Look for other opportunities to reduce the allocation rate
