export interface HttpClient {
	get(url: string | URL | Request): Promise<Response>
}
