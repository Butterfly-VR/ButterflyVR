extends Control


func send() -> void:
	var message:String = (get_child(0) as LineEdit).text.strip_escapes()
	if message.is_empty():
		return
	(get_child(0) as LineEdit).clear()
	(NetworkManager as NetNodeManager).network_message_send(message)
