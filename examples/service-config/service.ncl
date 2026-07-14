let LocalDevelopmentPort = 8080 in
{
  service = "payments-api",
  listen = {
    address = "127.0.0.1",
    port = LocalDevelopmentPort,
  },
  logging = {
    format = "json",
    include_request_ids = true,
  },
}
