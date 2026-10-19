locals {
  static_domains = [
    for name, cfg in local.stable_services : cfg.domain if try(cfg.static, false)
  ]
}

# Cloudflare caches the static hosts; nginx states each file's lifetime.
#
# Without the rule, Cloudflare's edge skips .wasm and .html, and the zone's
# browser TTL overwrites a revalidating file's `no-cache` with hours of
# freshness. Respecting the origin keeps the one policy nginx writes: hashed
# names immutable, everything else revalidated against the store-hash ETag,
# so a deploy needs no purge. The policy stays with nginx because only a regex
# tells a hashed name apart, and this zone's plan has none in rules.
resource "cloudflare_ruleset" "static_cache" {
  zone_id     = var.cloudflare_zone_id
  name        = "static hosts"
  description = "Cache the static hosts at the edge on the origin's terms"
  kind        = "zone"
  phase       = "http_request_cache_settings"

  rules {
    description = "Static hosts: cache everything, respect origin TTLs"
    expression  = "(http.host in {${join(" ", [for d in local.static_domains : "\"${d}\""])}})"
    action      = "set_cache_settings"
    enabled     = true

    action_parameters {
      cache = true

      edge_ttl {
        mode = "respect_origin"
      }

      browser_ttl {
        mode = "respect_origin"
      }
    }
  }
}
