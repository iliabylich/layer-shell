#include "ui/view_models/language_model.h"
#include <string.h>

struct _LanguageModel {
  GObject parent_instance;

  char *text;
};

G_DEFINE_TYPE(LanguageModel, language_model, G_TYPE_OBJECT)

enum {
  PROP_TEXT = 1,
  N_PROPERTIES,
};
static GParamSpec *properties[N_PROPERTIES] = {0};

static const char *to_language_text(const char *lang) {
  if (lang == NULL)
    return "??";
  if (strcmp(lang, "English (US)") == 0)
    return "EN";
  if (strcmp(lang, "Polish") == 0)
    return "PL";
  return "??";
}

static void language_model_get_property(GObject *object, guint property_id,
                                        GValue *value, GParamSpec *pspec) {
  LanguageModel *self = LANGUAGE_MODEL(object);
  switch (property_id) {
  case PROP_TEXT:
    g_value_set_string(value, self->text);
    break;
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void language_model_set_property(GObject *object, guint property_id,
                                        const GValue *value,
                                        GParamSpec *pspec) {
  LanguageModel *self = LANGUAGE_MODEL(object);
  switch (property_id) {
  case PROP_TEXT: {
    const char *text = to_language_text(g_value_get_string(value));
    g_free(self->text);
    self->text = g_strdup(text);
    g_object_notify_by_pspec(object, properties[PROP_TEXT]);
    break;
  }
  default:
    G_OBJECT_WARN_INVALID_PROPERTY_ID(object, property_id, pspec);
    break;
  }
}

static void language_model_finalize(GObject *object) {
  LanguageModel *self = LANGUAGE_MODEL(object);
  g_free(self->text);
  G_OBJECT_CLASS(language_model_parent_class)->finalize(object);
}

static void language_model_init(LanguageModel *self) { self->text = g_strdup("--"); }

static void language_model_class_init(LanguageModelClass *klass) {
  GObjectClass *object_class = G_OBJECT_CLASS(klass);
  object_class->get_property = language_model_get_property;
  object_class->set_property = language_model_set_property;
  object_class->finalize = language_model_finalize;

  properties[PROP_TEXT] =
      g_param_spec_string("text", NULL, NULL, "--",
                          G_PARAM_READWRITE | G_PARAM_EXPLICIT_NOTIFY);
  g_object_class_install_properties(object_class, N_PROPERTIES, properties);
}

LanguageModel *language_model_new(void) {
  return g_object_new(language_model_get_type(), NULL);
}
