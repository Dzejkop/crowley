# Crowley

A basic web scraper.

Exposes 3 api endpoints:

1. `/scrape/:url` where `:url` is a url encoded url to scrape
   Will scrape the provided url, following all the links it can find that lead to the same domain.
   Note that this request will only return a response once the scraping is done, which can take a long time.
2. `/count/:url` Counts all the indexed links for a given domain - domain will be extracted from the url.
3. `/list/:url` Lists all the indexed links for a given domain - domain will be extracted from the url.

# Setup
Run `./setup.sh` to create a `database.db` file with the required schema.

# Room for improvement
## Time of scraping and communication
A call to the `/scrape/:url` endpoint is likely to take a long time to run. I chose to leave it like it is as it's the easiest solution.

But for any reasonable production environment it would be necessary to setup some sort of control method, I see the following possible solutions:
1. A call to the scraping endpoint would schedule a task to scrape the given url, but the request would return a response immediately.
With this approach, another set of api endpoints would be needed to manage and check the status of scheduled scraping tasks.
2. An integration with a queue based communication system like RabbitMQ or Apache Kafka.

## Some edge cases and html weirdness
Right now crowley can handle a good portion of html it's given. It can handle any link it finds which as its `href` has an absolute link or a relative (relative to the domain) link that starts with `/`. It cannot handle link which are relative to the current location and it doesn't handle the [base](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/base) tag.

## Polishing
There's a lot of room for polish, for example:
1. HTTP header handling, right now even though the `/list/:url` route returns a json it's content type header is still `text/plain`.
2. Better db connection handling
3. A bunch of TODOs in code
