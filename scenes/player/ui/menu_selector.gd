extends TabContainer

func change_selected_menu(menu:int) -> void:
	if get_tab_count() > menu:
		current_tab = menu
