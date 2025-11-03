# Пример интеграции с Flutter WebApp

## Авторизация через Telegram

```dart
import 'package:dio/dio.dart';
import 'package:telegram_web_app/telegram_web_app.dart';

final dio = Dio(BaseOptions(
  baseUrl: 'https://your-domain.com',
  headers: {'Content-Type': 'application/json'},
));

// Авторизация через Telegram WebApp
Future<String> authenticateWithTelegram() async {
  try {
    // Получаем данные из Telegram WebApp SDK
    final initData = TelegramWebApp.initDataUnsafe;

    // Отправляем на сервер
    final response = await dio.post('/auth/telegram', data: initData);

    final token = response.data['token'] as String;
    final userId = response.data['user_id'] as String;

    // Сохраняем токен для последующих запросов
    await saveToken(token);

    return token;
  } catch (e) {
    print('Ошибка авторизации: $e');
    rethrow;
  }
}
```

## Обновление счёта

```dart
Future<void> updateScore(int score) async {
  final token = await getToken();

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
  } catch (e) {
    print('Ошибка обновления счёта: $e');
    rethrow;
  }
}
```

## Получение лидерборда

```dart
Future<List<LeaderboardEntry>> getLeaderboard() async {
  try {
    final response = await dio.get('/game/leaderboard');

    return (response.data as List)
        .map((e) => LeaderboardEntry.fromJson(e))
        .toList();
  } catch (e) {
    print('Ошибка получения лидерборда: $e');
    rethrow;
  }
}
```

## Вывод токенов

```dart
// Начать вывод
Future<String> startClaim(double amount) async {
  final token = await getToken();

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
  } catch (e) {
    print('Ошибка создания запроса: $e');
    rethrow;
  }
}

// Подтвердить вывод
Future<void> confirmClaim(String claimId) async {
  final token = await getToken();

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
  } catch (e) {
    print('Ошибка подтверждения вывода: $e');
    rethrow;
  }
}
```

## Полный пример

```dart
class GameApi {
  final Dio _dio;
  String? _token;

  GameApi(String baseUrl) : _dio = Dio(BaseOptions(baseUrl: baseUrl));

  Future<void> authenticate() async {
    final initData = TelegramWebApp.initDataUnsafe;
    final response = await _dio.post('/auth/telegram', data: initData);
    _token = response.data['token'] as String;
  }

  Options get _authOptions => Options(
    headers: {'Authorization': 'Bearer $_token'},
  );

  Future<void> updateScore(int score) async {
    await _dio.post(
      '/game/update_score',
      data: {'score': score},
      options: _authOptions,
    );
  }

  Future<List<dynamic>> getLeaderboard() async {
    final response = await _dio.get('/game/leaderboard');
    return response.data as List;
  }

  Future<String> startClaim(double amount) async {
    final response = await _dio.post(
      '/claim/start',
      data: {'amount': amount},
      options: _authOptions,
    );
    return response.data['claim_id'] as String;
  }

  Future<void> confirmClaim(String claimId) async {
    await _dio.post(
      '/claim/confirm',
      data: {'claim_id': claimId},
      options: _authOptions,
    );
  }
}
```

## Использование

```dart
void main() async {
  final api = GameApi('https://your-domain.com');

  // Авторизация
  await api.authenticate();

  // Обновить счёт
  await api.updateScore(1000);

  // Получить лидерборд
  final leaderboard = await api.getLeaderboard();
  print('Топ игроков: $leaderboard');

  // Вывод токенов
  final claimId = await api.startClaim(100.50);
  await api.confirmClaim(claimId);
}
```
