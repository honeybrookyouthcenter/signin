using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace YouthCenterSignIn.Logic.Tests
{
    [TestClass]
    public class TestBase
    {
        protected TestDataProvider DataProvider => (TestDataProvider)Data.DataProvider.Current;

        [TestInitialize]
        public void TestInit()
        {
            Data.DataProvider.Current = new TestDataProvider();
        }
    }
}
