using System;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Data;

namespace HBCCSignIn.Converters
{
    public class CountToVisibilityConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language) =>
            value is int intValue && intValue == (parameter as int? ?? 0) ? Visibility.Visible : Visibility.Collapsed;

        public object ConvertBack(object value, Type targetType, object parameter, string language) => throw new NotImplementedException();
    }
}
