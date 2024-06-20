export function evaluate(expr: string, document: Document, node?: Node) {
	return document.evaluate(expr, node ?? document, null, 0)
}

export function evaluateString(expr: string, document: Document, node?: Node) {
	return document.evaluate(expr, node ?? document, null, 2).stringValue.trim()
}
