# Привет!
В этой статье я расскажу, как можно написать свой плагин для Minecraft: Java Edition. Писать буду для ядра Paper, но сам плагин подойдет для его форков.

## Начало
Для этого вы должны установить JDK (я буду использовать 17 версию), и среду разработки. Код люди пишут где хотят, а я пишу в IntelliJ IDEA. Вы можете установить [Community Edition](https://www.jetbrains.com/idea/ "IntelliJ IDEA").

## Запуск IDE
После запуска IDE перейдите в раздел Plugins, и найдите там расширение «Minecraft Development».
С его помощью мы сможем создавать моды и плагины для Minecraft для любых ядер и мод-лоадеров.

Создаём свой первый проект.
Нажимаем на кнопку «New project», и в новом окне в «Generators» ищем «Minecraft». 

В поле «Name» введите название плагина, например «Plugin». По желанию можно выбрать место хранения в «Location».
Также можно создать репозиторий Git. Далее нажимаем «Plugin» (поскольку его мы и пишем), и выбираем ядро. Я всегда пишу плагины для Paper.
Они подходят и для его форков. Выбираем версию. Я пишу для 1.20.2.

В «Optional Settings» можно указать описание плагина, никнейм автора, ссылку на сайт плагина / разработчика, а также зависимости.
Зависимости это безумно важное поле, потому что без них плагин работать не сможет. Если вы пишите дополнение к какому-то плагину, то используйте «Soft Depend»,
а если пишите плагин с функционалом другого, указывайте его в «Depend».

«Group ID» это обычно домен разработчика или организации. Можно указать org.name.mc (адрес сервера), или ваш домен com.nickname.
«Artifact ID» это название плагина. «Version» это версия вашего плагина. Можно указать 1.0.

Теперь можно создать проект. Поначалу загрузка может показаться долгой, стоит подождать.
У нас появился скелет плагина, и мы теперь можем корректно с ним работать.

Мы напишем плагин на команду /discord, которая будет направлять пользователя в Discord-сервер.
```
// Главный класс плагина
public class DiscordCommand extends JavaPlugin {

    @Override
    public void onEnable() {
        // Регистрируем команду /discord
        this.getCommand("discord").setExecutor(new DiscordCommandExecutor());
    }
}
```

```
// Класс исполнитель команды
class DiscordCommandExecutor implements CommandExecutor {

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (label.equalsIgnoreCase("discord")) {
            if (sender instanceof Player) {
                Player player = (Player) sender;
                // Отправляем игроку ваш Discord URL или сообщение
                player.sendMessage(ChatColor.AQUA + "Присоединяйтесь к нашему Discord серверу: вашURL");
                return true;
            } else {
                sender.sendMessage("Эта команда может быть использована только игроком.");
                return true;
            }
        }
        return false;
    }
}
```

В целом, это достаточно просто. Библиотеки у вас должны подтянуться сами.