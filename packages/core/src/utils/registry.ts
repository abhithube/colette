import type { Scraper } from './scraper'

export class PluginRegistry<T, U> {
	constructor(private readonly plugins: Record<string, Scraper<T, U>>) {}

	register(host: string, scraper: Scraper<T, U>) {
		this.plugins[host] = scraper
	}

	load(host: string): Scraper<T, U> | undefined {
		return this.plugins[host]
	}
}
