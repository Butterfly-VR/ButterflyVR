extends Node
class_name ChatBoxManager

signal new_message_sent(message:Message)

var messages:Array[Message]

class Message:
	var player:int
	var text:String

func _physics_process(_delta: float) -> void:
	if (NetworkManager as NetNodeManager).has_message() and (NetworkManager as NetNodeManager).get_message_type() == 3:
		@warning_ignore("unsafe_call_argument")
		var player:int = (NetworkManager as NetNodeManager).get_message_player()
		var text:String = (NetworkManager as NetNodeManager).peek_message()[0]
		var message:Message = Message.new()
		message.player = player
		message.text = text
		new_message_sent.emit(message)
		messages.append(message)
		(NetworkManager as NetNodeManager).pop_message()
