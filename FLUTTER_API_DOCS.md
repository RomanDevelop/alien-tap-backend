# 📱 Документация API для Flutter приложения

## 🚀 Базовый URL

```
http://localhost:8000  # Для разработки
https://your-domain.com # Для production
```

## 🔐 Авторизация

### Шаг 1: Получение JWT токена

**Эндпоинт:** `POST /auth/telegram`

**Описание:** Авторизация через Telegram WebApp SDK. Проверяет подпись Telegram и возвращает JWT токен.

**Запрос:**
```dart
import 'package:dio/dio.dart';
import 'package:telegram_web_app/telegram_web_app.dart';

final dio = Dio(BaseOptions(
  baseUrl: 'http://localhost:8000',
  headers: {'Content-Type': 'application/json'},
));

Future<String> authenticate() async {
  try {
    // Получаем initData из Telegram WebApp SDK
    final initData = TelegramWebApp.initDataUnsafe;
    
    // Отправляем на сервер
    final response = await dio.post(
      '/auth/telegram',
      data: initData,
    );
    
    final token = response.data['token'] as String;
    final userId = response.data['user_id'] as String;
    
    // Сохраняем токен для последующих запросов
    await _saveToken(token);
    await _saveUserId(userId);
    
    return token;
  } catch (e) {
    print('Ошибка авторизации: $e');
    rethrow;
  }
}
```

**Ответ:**
```json
{
  "token": "eyJhbGc...",
  "user_id": "uuid-здесь"
}
```

**Ошибки:**
- `401 Unauthorized` - Неверная подпись Telegram
- `400 Bad Request` - Отсутствуют обязательные поля
- `500 Internal Server Error` - Ошибка сервера

---

## 🎮 Игровые эндпоинты

### 1. Обновление счёта

**Эндпоинт:** `POST /game/update_score`

**Описание:** Обновляет счёт пользователя. Сохраняет максимальное значение счёта.

**Запрос:**
```dart
Future<void> updateScore(int score) async {
  final token = await _getToken();
  
  try {
    final response = await dio.post(
      '/game/update_score',
      data: {'score': score},
      options: Options(
        headers: {
          'Authorization': 'Bearer $token',
        },
      ),
    );
    
    print('Счёт обновлён: ${response.data['score']}');
  } on DioException catch (e) {
    if (e.response?.statusCode == 401) {
      // Токен недействителен, нужно переавторизоваться
      await authenticate();
      await updateScore(score); // Повторяем запрос
    } else {
      print('Ошибка обновления счёта: ${e.message}');
      rethrow;
    }
  }
}
```

**Ответ:**
```json
{
  "success": true,
  "score": 1000
}
```

**Требования:**
- Заголовок `Authorization: Bearer <jwt_token>`

---

### 2. Получение лидерборда

**Эндпоинт:** `GET /game/leaderboard`

**Описание:** Возвращает топ-10 игроков по очкам.

**Запрос:**
```dart
class LeaderboardEntry {
  final String userId;
  final String? username;
  final String? firstName;
  final int score;
  
  LeaderboardEntry({
    required this.userId,
    this.username,
    this.firstName,
    required this.score,
  });
  
  factory LeaderboardEntry.fromJson(Map<String, dynamic> json) {
    return LeaderboardEntry(
      userId: json['user_id'] as String,
      username: json['username'] as String?,
      firstName: json['first_name'] as String?,
      score: json['score'] as int,
    );
  }
}

Future<List<LeaderboardEntry>> getLeaderboard() async {
  try {
    final response = await dio.get('/game/leaderboard');
    
    final List<dynamic> data = response.data as List;
    return data.map((json) => LeaderboardEntry.fromJson(json as Map<String, dynamic>)).toList();
  } catch (e) {
    print('Ошибка получения лидерборда: $e');
    rethrow;
  }
}
```

**Ответ:**
```json
[
  {
    "user_id": "uuid-1",
    "username": "player1",
    "first_name": "Player",
    "score": 5000
  },
  {
    "user_id": "uuid-2",
    "username": "player2",
    "first_name": "John",
    "score": 3500
  },
  ...
]
```

**Требования:** Не требует авторизации

---

## 💰 Эндпоинты вывода токенов

### 1. Начать вывод

**Эндпоинт:** `POST /claim/start`

**Описание:** Инициализирует запрос на вывод токенов.

**Запрос:**
```dart
Future<String> startClaim(double amount) async {
  final token = await _getToken();
  
  try {
    final response = await dio.post(
      '/claim/start',
      data: {'amount': amount},
      options: Options(
        headers: {
          'Authorization': 'Bearer $token',
        },
      ),
    );
    
    return response.data['claim_id'] as String;
  } on DioException catch (e) {
    if (e.response?.statusCode == 401) {
      await authenticate();
      return await startClaim(amount);
    } else {
      print('Ошибка создания запроса: ${e.message}');
      rethrow;
    }
  }
}
```

**Ответ:**
```json
{
  "claim_id": "uuid-claim",
  "status": "pending"
}
```

---

### 2. Подтвердить вывод

**Эндпоинт:** `POST /claim/confirm`

**Описание:** Подтверждает и завершает транзакцию вывода.

**Запрос:**
```dart
Future<void> confirmClaim(String claimId) async {
  final token = await _getToken();
  
  try {
    final response = await dio.post(
      '/claim/confirm',
      data: {'claim_id': claimId},
      options: Options(
        headers: {
          'Authorization': 'Bearer $token',
        },
      ),
    );
    
    print('Вывод подтверждён: ${response.data['status']}');
  } on DioException catch (e) {
    if (e.response?.statusCode == 401) {
      await authenticate();
      await confirmClaim(claimId);
    } else {
      print('Ошибка подтверждения: ${e.message}');
      rethrow;
    }
  }
}
```

**Ответ:**
```json
{
  "success": true,
  "status": "completed"
}
```

---

## 🔧 Полный класс API клиента

```dart
import 'package:dio/dio.dart';
import 'package:telegram_web_app/telegram_web_app.dart';
import 'package:shared_preferences/shared_preferences.dart';

class GameApi {
  final Dio _dio;
  String? _token;
  String? _userId;
  
  GameApi(String baseUrl) 
    : _dio = Dio(BaseOptions(
        baseUrl: baseUrl,
        headers: {'Content-Type': 'application/json'},
      ));
  
  // Инициализация и авторизация
  Future<void> initialize() async {
    final prefs = await SharedPreferences.getInstance();
    _token = prefs.getString('jwt_token');
    _userId = prefs.getString('user_id');
    
    if (_token == null) {
      await authenticate();
    }
  }
  
  // Авторизация через Telegram
  Future<String> authenticate() async {
    try {
      final initData = TelegramWebApp.initDataUnsafe;
      
      final response = await _dio.post('/auth/telegram', data: initData);
      
      _token = response.data['token'] as String;
      _userId = response.data['user_id'] as String;
      
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString('jwt_token', _token!);
      await prefs.setString('user_id', _userId!);
      
      return _token!;
    } on DioException catch (e) {
      if (e.response != null) {
        throw Exception('Ошибка авторизации: ${e.response!.data}');
      }
      throw Exception('Ошибка подключения: ${e.message}');
    }
  }
  
  // Получить заголовки с авторизацией
  Options get _authOptions => Options(
    headers: {'Authorization': 'Bearer $_token'},
  );
  
  // Обновить счёт
  Future<int> updateScore(int score) async {
    try {
      final response = await _dio.post(
        '/game/update_score',
        data: {'score': score},
        options: _authOptions,
      );
      
      return response.data['score'] as int;
    } on DioException catch (e) {
      if (e.response?.statusCode == 401) {
        await authenticate();
        return await updateScore(score);
      }
      throw Exception('Ошибка обновления счёта: ${e.message}');
    }
  }
  
  // Получить лидерборд
  Future<List<LeaderboardEntry>> getLeaderboard() async {
    try {
      final response = await _dio.get('/game/leaderboard');
      
      final List<dynamic> data = response.data as List;
      return data
          .map((json) => LeaderboardEntry.fromJson(json as Map<String, dynamic>))
          .toList();
    } catch (e) {
      throw Exception('Ошибка получения лидерборда: $e');
    }
  }
  
  // Начать вывод
  Future<String> startClaim(double amount) async {
    try {
      final response = await _dio.post(
        '/claim/start',
        data: {'amount': amount},
        options: _authOptions,
      );
      
      return response.data['claim_id'] as String;
    } on DioException catch (e) {
      if (e.response?.statusCode == 401) {
        await authenticate();
        return await startClaim(amount);
      }
      throw Exception('Ошибка создания запроса: ${e.message}');
    }
  }
  
  // Подтвердить вывод
  Future<void> confirmClaim(String claimId) async {
    try {
      await _dio.post(
        '/claim/confirm',
        data: {'claim_id': claimId},
        options: _authOptions,
      );
    } on DioException catch (e) {
      if (e.response?.statusCode == 401) {
        await authenticate();
        await confirmClaim(claimId);
      } else {
        throw Exception('Ошибка подтверждения: ${e.message}');
      }
    }
  }
  
  // Выход (очистка токена)
  Future<void> logout() async {
    _token = null;
    _userId = null;
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove('jwt_token');
    await prefs.remove('user_id');
  }
}

// Модель для лидерборда
class LeaderboardEntry {
  final String userId;
  final String? username;
  final String? firstName;
  final int score;
  
  LeaderboardEntry({
    required this.userId,
    this.username,
    this.firstName,
    required this.score,
  });
  
  factory LeaderboardEntry.fromJson(Map<String, dynamic> json) {
    return LeaderboardEntry(
      userId: json['user_id'] as String,
      username: json['username'] as String?,
      firstName: json['first_name'] as String?,
      score: json['score'] as int,
    );
  }
  
  String get displayName => 
    username ?? firstName ?? 'Игрок';
}
```

---

## 📦 Зависимости для pubspec.yaml

```yaml
dependencies:
  flutter:
    sdk: flutter
  dio: ^5.4.0              # HTTP клиент
  telegram_web_app: ^0.1.0 # Telegram WebApp SDK
  shared_preferences: ^2.2.2 # Хранение токенов
```

---

## 📝 Пример использования

```dart
void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  // Инициализация API
  final api = GameApi('http://localhost:8000');
  await api.initialize();
  
  runApp(MyApp(api: api));
}

class MyApp extends StatelessWidget {
  final GameApi api;
  
  const MyApp({required this.api, Key? key}) : super(key: key);
  
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Alien Tap Game',
      home: GameScreen(api: api),
    );
  }
}

class GameScreen extends StatefulWidget {
  final GameApi api;
  
  const GameScreen({required this.api, Key? key}) : super(key: key);
  
  @override
  State<GameScreen> createState() => _GameScreenState();
}

class _GameScreenState extends State<GameScreen> {
  int score = 0;
  List<LeaderboardEntry> leaderboard = [];
  
  @override
  void initState() {
    super.initState();
    _loadLeaderboard();
  }
  
  Future<void> _loadLeaderboard() async {
    try {
      final lb = await widget.api.getLeaderboard();
      setState(() => leaderboard = lb);
    } catch (e) {
      print('Ошибка загрузки лидерборда: $e');
    }
  }
  
  Future<void> _updateScore() async {
    setState(() => score++);
    
    try {
      await widget.api.updateScore(score);
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Ошибка сохранения счёта: $e')),
      );
    }
  }
  
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('Alien Tap')),
      body: Column(
        children: [
          Text('Счёт: $score', style: TextStyle(fontSize: 24)),
          ElevatedButton(
            onPressed: _updateScore,
            child: Text('Tap!'),
          ),
          Expanded(
            child: ListView.builder(
              itemCount: leaderboard.length,
              itemBuilder: (context, i) {
                final entry = leaderboard[i];
                return ListTile(
                  leading: Text('${i + 1}'),
                  title: Text(entry.displayName),
                  trailing: Text('${entry.score}'),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
```

---

## ⚠️ Обработка ошибок

### Типичные ошибки и их обработка:

**401 Unauthorized** - Токен истёк или недействителен
```dart
if (e.response?.statusCode == 401) {
  await api.authenticate(); // Переавторизация
  // Повторяем запрос
}
```

**400 Bad Request** - Неверные данные
```dart
if (e.response?.statusCode == 400) {
  final error = e.response!.data['error'] as String;
  // Показать пользователю ошибку
}
```

**500 Internal Server Error** - Ошибка сервера
```dart
if (e.response?.statusCode == 500) {
  // Показать сообщение об ошибке сервера
  // Возможно, повторить запрос позже
}
```

---

## 🔒 Безопасность

1. **Храните JWT токен безопасно**
   - Используйте `shared_preferences` для локального хранения
   - Никогда не коммитьте токены в Git
   
2. **Обрабатывайте истечение токена**
   - Токен действителен 24 часа
   - При 401 ошибке - переавторизуйтесь
   
3. **Проверяйте данные перед отправкой**
   - Валидируйте score >= 0
   - Валидируйте amount > 0 для claims

---

## 📊 Схема работы приложения

```
1. Запуск приложения
   ↓
2. Проверка сохранённого токена
   ↓
3. Если токена нет → Авторизация через Telegram
   ↓
4. Сохранение токена
   ↓
5. Загрузка лидерборда
   ↓
6. Игровой процесс → Обновление счёта
   ↓
7. Вывод токенов (опционально)
```

---

## 🧪 Тестирование

Для тестирования без Telegram WebApp SDK используйте утилиту:

```bash
# Сгенерировать тестовый hash
cargo run --bin generate_hash

# Использовать полученный JSON в Postman/Thunder Client
```

---

## 📚 Дополнительные ресурсы

- **Backend README:** `README.md`
- **Быстрый старт:** `QUICKSTART.md`
- **Примеры интеграции:** `INTEGRATION_EXAMPLE.md`

---

## ✅ Чек-лист интеграции

- [ ] Установлены зависимости (dio, telegram_web_app, shared_preferences)
- [ ] Создан класс GameApi
- [ ] Реализована авторизация через Telegram
- [ ] Реализовано обновление счёта
- [ ] Реализован лидерборд
- [ ] Реализован вывод токенов (если нужно)
- [ ] Обработка ошибок (401, 400, 500)
- [ ] Сохранение и восстановление токена
- [ ] Тестирование всех эндпоинтов

---

**Готово к интеграции!** 🚀

