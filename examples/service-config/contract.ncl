{
  service | String,
  listen = {
    address | String,
    port | Number,
  },
  logging = {
    format | String,
    include_request_ids | Bool,
  },
}
