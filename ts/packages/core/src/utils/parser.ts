export interface ResponseParser<T> {
	parse(res: Response): Promise<T>
}
