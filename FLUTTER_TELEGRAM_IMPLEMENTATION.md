# 📱 Инструкции для Flutter: Правильная реализация авторизации через Telegram

## 🎯 Цель

Реализовать авторизацию через Telegram WebApp SDK **строго по официальной документации**, чтобы бэкенд мог корректно проверить подпись Telegram.

## 📚 Официальная документация

**Ссылка:** https://core.telegram.org/bots/webapps#validating-data-received

**Ключевой момент:** Бэкенд должен получить **оригинальную строку `initData`** от Telegram WebApp SDK, а не пересозданный JSON объект.

## ✅ Правильная реализация

### Шаг 1: Установка пакета

```yaml
# pubspec.yaml
dependencies:
  telegram_web_app: ^0.1.0 # или актуальная версия
  dio: ^5.0.0
```

### Шаг 2: Получение initData (ПРАВИЛЬНО)

```dart
import 'package:telegram_web_app/telegram_web_app.dart';

// ✅ ПРАВИЛЬНО: Получаем оригинальную строку initData
String? getInitDataString() {
  try {
    // Используем initDataUnsafe для получения строки
    final initData = TelegramWebApp.initDataUnsafe;

    if (initData == null || initData.isEmpty) {
      print('⚠️ initData пустой или null');
      return null;
    }

    print('✅ Получен initData (длина: ${initData.length})');
    print('   Первые 100 символов: ${initData.substring(0, initData.length > 100 ? 100 : initData.length)}');

    return initData;
  } catch (e) {
    print('❌ Ошибка получения initData: $e');
    return null;
  }
}
```

### Шаг 3: Отправка на бэкенд (ПРАВИЛЬНО)

```dart
import 'package:dio/dio.dart';

Future<Map<String, dynamic>?> authenticateWithTelegram() async {
  try {
    // ✅ ПРАВИЛЬНО: Получаем оригинальную строку initData
    final initDataString = getInitDataString();

    if (initDataString == null) {
      throw Exception('initData не получен от Telegram');
    }

    // ✅ ПРАВИЛЬНО: Отправляем initData как строку в JSON
    final dio = Dio(BaseOptions(
      baseUrl: 'http://localhost:8000', // или ваш URL
      headers: {'Content-Type': 'application/json'},
    ));

    final response = await dio.post(
      '/auth/telegram',
      data: {
        'initData': initDataString,  // ✅ Отправляем как строку!
      },
    );

    if (response.statusCode == 200) {
      final token = response.data['token'] as String;
      final userId = response.data['user_id'] as String;

      print('✅ Авторизация успешна');
      print('   Token: ${token.substring(0, 20)}...');
      print('   User ID: $userId');

      // Сохраняем токен
      await _saveToken(token);
      await _saveUserId(userId);

      return {
        'token': token,
        'user_id': userId,
      };
    } else {
      throw Exception('Ошибка авторизации: ${response.statusCode}');
    }
  } catch (e) {
    print('❌ Ошибка авторизации: $e');
    rethrow;
  }
}
```

## ❌ НЕПРАВИЛЬНАЯ реализация (НЕ ДЕЛАТЬ ТАК!)

### ❌ Ошибка 1: Парсинг initData на клиенте

```dart
// ❌ НЕПРАВИЛЬНО: Не парсите initData на клиенте!
final initData = TelegramWebApp.initDataUnsafe;
final parsed = Uri.splitQueryString(initData);  // ❌ НЕ ДЕЛАТЬ!
final userJson = jsonDecode(parsed['user']!);   // ❌ НЕ ДЕЛАТЬ!

// ❌ НЕПРАВИЛЬНО: Не пересоздавайте JSON!
await dio.post('/auth/telegram', data: {
  'hash': parsed['hash'],
  'auth_date': parsed['auth_date'],
  'user': userJson,  // ❌ Порядок ключей может быть другим!
});
```

**Проблема:** При парсинге и пересоздании JSON теряется оригинальный порядок ключей, что приводит к неверной проверке подписи на бэкенде.

### ❌ Ошибка 2: Использование initDataUnsafe как объекта

```dart
// ❌ НЕПРАВИЛЬНО: initDataUnsafe - это строка, а не объект!
final initData = TelegramWebApp.initDataUnsafe;
final user = initData.user;  // ❌ НЕ ДЕЛАТЬ!

await dio.post('/auth/telegram', data: {
  'user': user.toJson(),  // ❌ Порядок ключей будет другим!
});
```

### ❌ Ошибка 3: Декодирование URL-encoded значений

```dart
// ❌ НЕПРАВИЛЬНО: Не декодируйте значения перед отправкой!
final initData = TelegramWebApp.initDataUnsafe;
final decoded = Uri.decodeComponent(initData);  // ❌ НЕ ДЕЛАТЬ!

await dio.post('/auth/telegram', data: {
  'initData': decoded,  // ❌ Бэкенд ожидает URL-encoded строку!
});
```

## 📋 Полный пример правильной реализации

```dart
// lib/services/telegram_auth_service.dart
import 'package:dio/dio.dart';
import 'package:telegram_web_app/telegram_web_app.dart';
import 'package:shared_preferences/shared_preferences.dart';

class TelegramAuthService {
  final Dio _dio;
  static const String _tokenKey = 'jwt_token';
  static const String _userIdKey = 'user_id';

  TelegramAuthService({String? baseUrl})
      : _dio = Dio(BaseOptions(
          baseUrl: baseUrl ?? 'http://localhost:8000',
          headers: {'Content-Type': 'application/json'},
        ));

  /// Получает оригинальную строку initData от Telegram WebApp SDK
  String? getInitDataString() {
    try {
      final initData = TelegramWebApp.initDataUnsafe;

      if (initData == null || initData.isEmpty) {
        print('⚠️ initData пустой или null');
        return null;
      }

      print('✅ Получен initData от Telegram');
      print('   Длина: ${initData.length} символов');

      return initData;
    } catch (e) {
      print('❌ Ошибка получения initData: $e');
      return null;
    }
  }

  /// Авторизация через Telegram
  Future<AuthResult> authenticate() async {
    try {
      // ✅ Получаем оригинальную строку initData
      final initDataString = getInitDataString();

      if (initDataString == null) {
        throw AuthException('initData не получен от Telegram WebApp SDK');
      }

      // ✅ Отправляем initData как строку
      final response = await _dio.post(
        '/auth/telegram',
        data: {
          'initData': initDataString,  // ✅ Строка, не объект!
        },
      );

      if (response.statusCode == 200) {
        final token = response.data['token'] as String;
        final userId = response.data['user_id'] as String;

        // Сохраняем токен и user_id
        await _saveToken(token);
        await _saveUserId(userId);

        return AuthResult(
          token: token,
          userId: userId,
          success: true,
        );
      } else {
        throw AuthException('Ошибка авторизации: ${response.statusCode}');
      }
    } on DioException catch (e) {
      if (e.response?.statusCode == 401) {
        throw AuthException('Неверная подпись Telegram');
      } else if (e.response?.statusCode == 400) {
        throw AuthException('Отсутствуют обязательные поля');
      } else {
        throw AuthException('Ошибка сети: ${e.message}');
      }
    } catch (e) {
      throw AuthException('Ошибка авторизации: $e');
    }
  }

  /// Сохраняет JWT токен
  Future<void> _saveToken(String token) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_tokenKey, token);
  }

  /// Сохраняет user_id
  Future<void> _saveUserId(String userId) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_userIdKey, userId);
  }

  /// Получает сохраненный токен
  Future<String?> getToken() async {
    final prefs = await SharedPreferences.getInstance();
    return prefs.getString(_tokenKey);
  }

  /// Получает сохраненный user_id
  Future<String?> getUserId() async {
    final prefs = await SharedPreferences.getInstance();
    return prefs.getString(_userIdKey);
  }

  /// Проверяет, авторизован ли пользователь
  Future<bool> isAuthenticated() async {
    final token = await getToken();
    return token != null && token.isNotEmpty;
  }

  /// Выход из системы
  Future<void> logout() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_tokenKey);
    await prefs.remove(_userIdKey);
  }
}

class AuthResult {
  final String token;
  final String userId;
  final bool success;

  AuthResult({
    required this.token,
    required this.userId,
    required this.success,
  });
}

class AuthException implements Exception {
  final String message;
  AuthException(this.message);

  @override
  String toString() => 'AuthException: $message';
}
```

## 🔧 Использование в приложении

```dart
// lib/screens/auth_screen.dart
import 'package:flutter/material.dart';
import '../services/telegram_auth_service.dart';

class AuthScreen extends StatefulWidget {
  @override
  _AuthScreenState createState() => _AuthScreenState();
}

class _AuthScreenState extends State<AuthScreen> {
  final TelegramAuthService _authService = TelegramAuthService(
    baseUrl: 'http://localhost:8000', // или ваш URL
  );

  bool _isLoading = false;
  String? _error;

  Future<void> _authenticate() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    try {
      final result = await _authService.authenticate();

      if (result.success) {
        // Переход на главный экран
        Navigator.pushReplacementNamed(context, '/home');
      }
    } on AuthException catch (e) {
      setState(() {
        _error = e.message;
      });
    } finally {
      setState(() {
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            if (_error != null)
              Padding(
                padding: EdgeInsets.all(16),
                child: Text(
                  _error!,
                  style: TextStyle(color: Colors.red),
                ),
              ),
            ElevatedButton(
              onPressed: _isLoading ? null : _authenticate,
              child: _isLoading
                  ? CircularProgressIndicator()
                  : Text('Войти через Telegram'),
            ),
          ],
        ),
      ),
    );
  }
}
```

## 📝 Чек-лист правильной реализации

- [ ] Используется `TelegramWebApp.initDataUnsafe` для получения строки
- [ ] `initData` отправляется как **строка**, а не как объект
- [ ] Не выполняется парсинг `initData` на клиенте
- [ ] Не пересоздаются JSON объекты из `initData`
- [ ] Не декодируются URL-encoded значения
- [ ] `initData` отправляется в поле `initData` (с заглавной буквы D)
- [ ] Обрабатываются ошибки авторизации (401, 400, 500)
- [ ] Токен сохраняется для последующих запросов

## ⚠️ Важные замечания

1. **initData - это строка**, а не объект. Не парсите её на клиенте!
2. **Порядок ключей критичен** - бэкенд проверяет подпись по оригинальной строке
3. **URL-encoding сохраняется** - не декодируйте значения перед отправкой
4. **Проверка подписи на сервере** - клиент только передает данные

## 🔍 Отладка

### Проверка, что initData получен:

```dart
final initData = TelegramWebApp.initDataUnsafe;
print('initData: $initData');
print('Длина: ${initData?.length ?? 0}');
print('Содержит hash: ${initData?.contains('hash=') ?? false}');
print('Содержит user: ${initData?.contains('user=') ?? false}');
```

### Проверка запроса к бэкенду:

```dart
// Включите логирование Dio
_dio.interceptors.add(LogInterceptor(
  requestBody: true,
  responseBody: true,
));
```

## 📚 Дополнительные ресурсы

- [Официальная документация Telegram](https://core.telegram.org/bots/webapps#validating-data-received)
- [Telegram WebApp SDK для Flutter](https://pub.dev/packages/telegram_web_app)
- [Dio документация](https://pub.dev/packages/dio)

## ✅ Результат

После правильной реализации:

- ✅ Бэкенд получит оригинальную строку `initData`
- ✅ Проверка подписи будет работать корректно
- ✅ Авторизация пройдет успешно
- ✅ Не будет необходимости в перестановках ключей на бэкенде
