export type Feed = {
	link: string
	title: string
	entries: Entry[]
}

export type Entry = {
	link: string
	title: string
	published?: string
	description?: string
	author?: string
	thumbnail?: string
}
