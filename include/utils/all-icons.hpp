#define _CONCAT(a, b) a##b
#define CONCAT(a, b) _CONCAT(a, b)
#define CONCAT3(a, b, c) CONCAT(a, CONCAT(b, c))

#ifndef X
#define X(name)
#endif

X(change_theme)
X(download)
X(foggy)
X(partly_cloudy)
X(power)
X(question_mark)
X(rainy)
X(snowy)
X(sunny)
X(thunderstorm)
X(upload)
X(wifi)
